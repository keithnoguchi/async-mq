// SPDX-License-Identifier: APACHE-2.0 AND MIT
//! [ProducerBuilder], [Producer] structs, and [ProducerHandler] traits
//!
//! [ProducerBuilder]: struct.ProducerBuilder.html
//! [Producer]: struct.Producer.html
//! [ProducerHandler]: trait.ProducerHandler.html
use async_trait::async_trait;
use futures_util::stream::StreamExt;
use lapin;

/// A [non-consuming] [Producer] builder.
///
/// [Producer]: struct.Producer.html
/// [non-consuming]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html#non-consuming-builders-(preferred):
#[derive(Clone)]
pub struct ProducerBuilder {
    conn: crate::Connection,
    ex: String,
    queue: String,
    queue_opts: lapin::options::QueueDeclareOptions,
    field_table: lapin::types::FieldTable,
    tx_props: lapin::BasicProperties,
    tx_opts: lapin::options::BasicPublishOptions,
    rx_opts: lapin::options::BasicConsumeOptions,
    ack_opts: lapin::options::BasicAckOptions,
    nack_opts: lapin::options::BasicNackOptions,
    handler: Box<dyn crate::ProducerHandler + Send>,
}

impl ProducerBuilder {
    pub fn new(conn: crate::Connection) -> Self {
        Self {
            conn,
            ex: String::from(""),
            queue: String::from(""),
            queue_opts: lapin::options::QueueDeclareOptions::default(),
            field_table: lapin::types::FieldTable::default(),
            tx_props: lapin::BasicProperties::default(),
            tx_opts: lapin::options::BasicPublishOptions::default(),
            rx_opts: lapin::options::BasicConsumeOptions::default(),
            ack_opts: lapin::options::BasicAckOptions::default(),
            nack_opts: lapin::options::BasicNackOptions::default(),
            handler: Box::new(crate::produce::NoopPeeker {}),
        }
    }
    pub fn exchange(&mut self, exchange: String) -> &mut Self {
        self.ex = exchange;
        self
    }
    pub fn queue(&mut self, queue: String) -> &mut Self {
        self.queue = queue;
        self
    }
    /// Use the provided [ProducerHandler] trait object.
    ///
    /// [ProducerHandler]: trait.ProducerHandler.html
    pub fn with_handler(&mut self, handler: Box<dyn crate::ProducerHandler + Send>) -> &mut Self {
        self.handler = handler;
        self
    }
    pub async fn build(&self) -> crate::Result<Producer> {
        let tx = self
            .conn
            .channel(
                &self.queue,
                self.queue_opts.clone(),
                self.field_table.clone(),
            )
            .await
            .map(|(ch, _)| ch)?;
        let opts = lapin::options::QueueDeclareOptions {
            exclusive: true,
            auto_delete: true,
            ..self.queue_opts.clone()
        };
        let (rx, q) = self
            .conn
            .channel("", opts, self.field_table.clone())
            .await?;
        let consume = rx
            .basic_consume(
                &q,
                "producer",
                self.rx_opts.clone(),
                self.field_table.clone(),
            )
            .await
            .map_err(crate::Error::from)?;
        Ok(Producer {
            tx,
            rx,
            consume,
            ex: self.ex.clone(),
            queue: self.queue.clone(),
            tx_props: self.tx_props.clone(),
            rx_props: self.tx_props.clone().with_reply_to(q.name().clone()),
            tx_opts: self.tx_opts.clone(),
            ack_opts: self.ack_opts.clone(),
            nack_opts: self.nack_opts.clone(),
            handler: self.handler.clone(),
        })
    }
}

/// A zero-cost message producer over [lapin::Channel].
///
/// [lapin::Channel]: https://docs.rs/lapin/latest/lapin/struct.Channel.html
pub struct Producer {
    tx: lapin::Channel,
    rx: lapin::Channel,
    consume: lapin::Consumer,
    ex: String,
    queue: String,
    tx_props: lapin::BasicProperties,
    rx_props: lapin::BasicProperties,
    tx_opts: lapin::options::BasicPublishOptions,
    ack_opts: lapin::options::BasicAckOptions,
    nack_opts: lapin::options::BasicNackOptions,
    handler: Box<dyn crate::ProducerHandler + Send>,
}

impl Producer {
    /// Use the provided [ProducerHandler] trait object.
    ///
    /// [ProducerHandler]: trait.ProducerHandler.html
    pub fn with_handler(&mut self, handler: Box<dyn crate::ProducerHandler + Send>) -> &mut Self {
        self.handler = handler;
        self
    }
    pub async fn publish(&mut self, msg: Vec<u8>) -> crate::Result<()> {
        self.tx
            .basic_publish(
                &self.ex,
                &self.queue,
                self.tx_opts.clone(),
                msg,
                self.tx_props.clone(),
            )
            .await
            .map_err(crate::Error::from)?;
        Ok(())
    }
    pub async fn rpc(&mut self, msg: Vec<u8>) -> crate::Result<Vec<u8>> {
        self.tx
            .basic_publish(
                &self.ex,
                &self.queue,
                self.tx_opts.clone(),
                msg,
                self.rx_props.clone(),
            )
            .await
            .map_err(crate::Error::from)?;
        if let Some(msg) = self.consume.next().await {
            match msg {
                Ok(msg) => return self.recv(msg).await,
                Err(err) => return Err(crate::Error::from(err)),
            }
        }
        Ok(vec![])
    }
    async fn recv(&mut self, msg: lapin::message::Delivery) -> crate::Result<Vec<u8>> {
        let delivery_tag = msg.delivery_tag;
        match self.handler.peek(msg.data).await {
            Ok(msg) => {
                self.rx
                    .basic_ack(delivery_tag, self.ack_opts.clone())
                    .await
                    .map_err(crate::Error::from)?;
                Ok(msg)
            }
            Err(_err) => {
                self.rx
                    .basic_nack(delivery_tag, self.nack_opts.clone())
                    .await
                    .map_err(crate::Error::from)?;
                Ok(vec![])
            }
        }
    }
}

/// A trait to extend the [Producer] capability.
///
/// [Producer]: struct.Producer.html
#[async_trait]
pub trait ProducerHandler {
    async fn peek(&mut self, msg: Vec<u8>) -> crate::Result<Vec<u8>>;
    fn box_clone(&self) -> Box<dyn ProducerHandler + Send>;
}

// https://users.rust-lang.org/t/solved-is-it-possible-to-clone-a-boxed-trait-object/1714/6
impl Clone for Box<dyn ProducerHandler + Send> {
    fn clone(&self) -> Box<dyn ProducerHandler + Send> {
        self.box_clone()
    }
}

/// A default [ProducerHandler] implementor that just nothing to do
/// with the received message.
///
/// [ProducerHandler]: trait.ProducerHandler.html
#[derive(Clone)]
pub struct NoopPeeker;

#[async_trait]
impl ProducerHandler for NoopPeeker {
    async fn peek(&mut self, msg: Vec<u8>) -> crate::Result<Vec<u8>> {
        Ok(msg)
    }
    fn box_clone(&self) -> Box<dyn ProducerHandler + Send> {
        Box::new((*self).clone())
    }
}
