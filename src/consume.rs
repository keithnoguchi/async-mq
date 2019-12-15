// SPDX-License-Identifier: APACHE-2.0 AND MIT
//! [ConsumerBuilder], [Consumer] structs, and [ConsumerHandler] trait
//!
//! [ConsumerBuilder]: struct.ConsumerBuilder.html
//! [Consumer]: struct.Consumer.html
//! [ConsumerHandler]: trait.ConsumerHandler.html
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use lapin;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A [non-consuming] [Consumer] builder.
///
/// [Consumer]: struct.Consumer.html
/// [non-consuming]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html#non-consuming-builders-(preferred):
#[derive(Clone)]
pub struct ConsumerBuilder {
    conn: crate::Connection,
    ex: String,
    queue: String,
    queue_opts: lapin::options::QueueDeclareOptions,
    field_table: lapin::types::FieldTable,
    tx_props: lapin::BasicProperties,
    tx_opts: lapin::options::BasicPublishOptions,
    rx_opts: lapin::options::BasicConsumeOptions,
    ack_opts: lapin::options::BasicAckOptions,
    rej_opts: lapin::options::BasicRejectOptions,
    handler: Box<dyn crate::ConsumerHandler + Send>,
}

impl ConsumerBuilder {
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
            rej_opts: lapin::options::BasicRejectOptions::default(),
            handler: Box::new(EchoMessenger {}),
        }
    }
    /// Override the default exchange name.
    pub fn exchange(&mut self, exchange: String) -> &mut Self {
        self.ex = exchange;
        self
    }
    /// Override the default queue name.
    pub fn queue(&mut self, queue: String) -> &mut Self {
        self.queue = queue;
        self
    }
    /// Use the provided [ConsumerHandler] trait object.
    ///
    /// [ConsumerHandler]: trait.ConsumerHandler.html
    pub fn with_handler(&mut self, handler: Box<dyn crate::ConsumerHandler + Send>) -> &mut Self {
        self.handler = handler;
        self
    }
    pub async fn build(&self) -> crate::Result<Consumer> {
        let (ch, q) = self
            .conn
            .channel(
                &self.queue,
                self.queue_opts.clone(),
                self.field_table.clone(),
            )
            .await?;
        let consume = ch
            .clone()
            .basic_consume(
                &q,
                "consumer",
                self.rx_opts.clone(),
                self.field_table.clone(),
            )
            .await
            .map_err(crate::Error::from)?;
        Ok(Consumer {
            ch,
            consume,
            tx_props: self.tx_props.clone(),
            tx_opts: self.tx_opts.clone(),
            ack_opts: self.ack_opts.clone(),
            rej_opts: self.rej_opts.clone(),
            handler: self.handler.clone(),
        })
    }
}

/// A zero-cost [lapin::Consumer] abstruction type.
///
/// [lapin::Consumer]: https://docs.rs/lapin/latest/lapin/struct.Consumer.html
pub struct Consumer {
    ch: lapin::Channel,
    consume: lapin::Consumer,
    tx_props: lapin::BasicProperties,
    tx_opts: lapin::options::BasicPublishOptions,
    ack_opts: lapin::options::BasicAckOptions,
    rej_opts: lapin::options::BasicRejectOptions,
    handler: Box<dyn crate::ConsumerHandler + Send>,
}

impl Consumer {
    pub async fn response(&mut self, req: &crate::Message, resp: &[u8]) -> crate::Result<()> {
        let delivery_tag = req.0.delivery_tag;
        let reply_to = req.0.properties.reply_to();
        if let Some(reply_to) = reply_to {
            self.send(reply_to.as_str(), resp).await?;
        }
        self.ch
            .basic_ack(delivery_tag, self.ack_opts.clone())
            .await
            .map_err(crate::Error::from)?;
        Ok(())
    }
    pub async fn reject(&mut self, req: crate::Message) -> crate::Result<()> {
        let delivery_tag = req.0.delivery_tag;
        self.ch
            .basic_reject(delivery_tag, self.rej_opts.clone())
            .await
            .map_err(crate::Error::from)?;
        Ok(())
    }
    /// Use the provided [ConsumerHandler] trait object.
    ///
    /// [ConsumerHandler]: trait.ConsumerHandler.html
    pub fn with_handler(&mut self, handler: Box<dyn crate::ConsumerHandler + Send>) -> &mut Self {
        self.handler = handler;
        self
    }
    pub async fn run(&mut self) -> crate::Result<()> {
        while let Some(msg) = self.consume.next().await {
            match msg {
                Ok(msg) => self.recv(msg).await?,
                Err(err) => return Err(crate::Error::from(err)),
            }
        }
        Ok(())
    }
    /// Transfer the received message to [ConsumerHandler] and acknowledge
    /// it to the broker, or nack in case [ConsumerHandler] implementor returns
    /// error.  In case of the message contains `reply_to` field, it will
    /// send back to the response queue specified in `reply_to` field in
    /// the message.
    ///
    /// [ConsumerHandler]: trait.ConsumerHandler.html
    async fn recv(&mut self, msg: lapin::message::Delivery) -> crate::Result<()> {
        let delivery_tag = msg.delivery_tag;
        let reply_to = msg.properties.reply_to();
        match self.handler.recv(msg.data).await {
            Ok(msg) => {
                if let Some(reply_to) = reply_to {
                    self.send(reply_to.as_str(), &msg).await?;
                }
                self.ch
                    .basic_ack(delivery_tag, self.ack_opts.clone())
                    .await
                    .map_err(crate::Error::from)?
            }
            Err(_err) => self
                .ch
                .basic_reject(delivery_tag, self.rej_opts.clone())
                .await
                .map_err(crate::Error::from)?,
        }
        Ok(())
    }
    async fn send(&mut self, queue: &str, msg: &[u8]) -> crate::Result<()> {
        self.ch
            .basic_publish(
                "",
                &queue,
                self.tx_opts.clone(),
                msg.to_vec(),
                self.tx_props.clone(),
            )
            .await
            .map_err(crate::Error::from)?;
        Ok(())
    }
}

impl Stream for Consumer {
    type Item = Result<crate::Message, crate::Error>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let c = &mut self.consume;
        let c = Pin::new(c);
        match c.poll_next(cx) {
            Poll::Ready(Some(Ok(msg))) => Poll::Ready(Some(Ok(crate::Message(msg)))),
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(err.into()))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A trait to extend the [Consumer] capability.
///
/// [Consumer]: struct.Consumer.html
#[async_trait]
pub trait ConsumerHandler {
    /// Async method to transfer the message to [ConsumerHandler] implementor.
    ///
    /// [ConsumerHandler]: trait.ConsumerHandler.html
    async fn recv(&mut self, msg: Vec<u8>) -> crate::Result<Vec<u8>>;
    fn boxed_clone(&self) -> Box<dyn ConsumerHandler + Send>;
}

// https://users.rust-lang.org/t/solved-is-it-possible-to-clone-a-boxed-trait-object/1714/6
impl Clone for Box<dyn ConsumerHandler + Send> {
    fn clone(&self) -> Box<dyn ConsumerHandler + Send> {
        self.boxed_clone()
    }
}

/// A default [ConsumerHandler] implementor that echoes back the received message.
///
/// [ConsumerHandler]: trait.ConsumerHandler.html
#[derive(Clone)]
pub struct EchoMessenger;

#[async_trait]
impl ConsumerHandler for EchoMessenger {
    /// Echoe back the received message.
    async fn recv(&mut self, msg: Vec<u8>) -> crate::Result<Vec<u8>> {
        Ok(msg)
    }
    fn boxed_clone(&self) -> Box<dyn ConsumerHandler + Send> {
        Box::new((*self).clone())
    }
}
