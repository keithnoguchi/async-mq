// SPDX-License-Identifier: APACHE-2.0 AND MIT
//! `ProducerBuilder` and `Producer` structs
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
    kind: lapin::ExchangeKind,
    ex_opts: lapin::options::ExchangeDeclareOptions,
    queue_opts: lapin::options::QueueDeclareOptions,
    bind_opts: lapin::options::QueueBindOptions,
    field_table: lapin::types::FieldTable,
    tx_props: lapin::BasicProperties,
    tx_opts: lapin::options::BasicPublishOptions,
    rx_opts: lapin::options::BasicConsumeOptions,
    ack_opts: lapin::options::BasicAckOptions,
    rej_opts: lapin::options::BasicRejectOptions,
    nack_opts: lapin::options::BasicNackOptions,
    peeker: Box<dyn crate::MessagePeek + Send + Sync>,
}

impl ProducerBuilder {
    pub fn new(conn: crate::Connection) -> Self {
        Self {
            conn,
            ex: String::from(crate::DEFAULT_EXCHANGE),
            queue: String::from(crate::DEFAULT_QUEUE),
            kind: lapin::ExchangeKind::Direct,
            ex_opts: lapin::options::ExchangeDeclareOptions::default(),
            queue_opts: lapin::options::QueueDeclareOptions::default(),
            bind_opts: lapin::options::QueueBindOptions::default(),
            field_table: lapin::types::FieldTable::default(),
            tx_props: lapin::BasicProperties::default(),
            tx_opts: lapin::options::BasicPublishOptions::default(),
            rx_opts: lapin::options::BasicConsumeOptions::default(),
            ack_opts: lapin::options::BasicAckOptions::default(),
            rej_opts: lapin::options::BasicRejectOptions::default(),
            nack_opts: lapin::options::BasicNackOptions::default(),
            peeker: Box::new(crate::message::NoopPeeker {}),
        }
    }
    /// Specify the exchange name.
    pub fn exchange(&mut self, exchange: &str) -> &mut Self {
        self.ex = exchange.to_string();
        self
    }
    /// Specify the queue name.
    pub fn queue(&mut self, queue: &str) -> &mut Self {
        self.queue = queue.to_string();
        self
    }
    /// Use the provided [MessagePeek] trait object.
    ///
    /// [MessagePeek]: ../message/trait.MessagePeek.html
    pub fn with_peeker(&mut self, peeker: Box<dyn crate::MessagePeek + Send + Sync>) -> &mut Self {
        self.peeker = peeker;
        self
    }
    pub async fn build(&self) -> crate::Result<Producer> {
        let tx = self.conn.channel().await?;
        let queue_opts = lapin::options::QueueDeclareOptions {
            exclusive: true,
            auto_delete: true,
            ..self.queue_opts.clone()
        };
        let opts = crate::client::QueueOptions {
            kind: self.kind.clone(),
            ex_opts: self.ex_opts.clone(),
            ex_field: self.field_table.clone(),
            queue_opts,
            queue_field: self.field_table.clone(),
            bind_opts: self.bind_opts.clone(),
            bind_field: self.field_table.clone(),
        };
        let (rx, q) = self
            .conn
            .queue(&self.ex, crate::EPHEMERAL_QUEUE, opts)
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
            rej_opts: self.rej_opts.clone(),
            nack_opts: self.nack_opts.clone(),
            peeker: self.peeker.clone(),
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
    rej_opts: lapin::options::BasicRejectOptions,
    nack_opts: lapin::options::BasicNackOptions,
    peeker: Box<dyn crate::MessagePeek + Send>,
}

impl Producer {
    /// Use the provided [MessagePeek] trait object.
    ///
    /// [MessagePeek]: ../message/trait.MessagePeek.html
    pub fn with_peeker(&mut self, peeker: Box<dyn crate::MessagePeek + Send + Sync>) -> &mut Self {
        self.peeker = peeker;
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
                Ok(msg) => return self.recv(&crate::Message::new(msg)).await,
                Err(err) => return Err(crate::Error::from(err)),
            }
        }
        Ok(vec![])
    }
    async fn recv(&mut self, msg: &crate::Message) -> crate::Result<Vec<u8>> {
        match self.peeker.peek(msg).await {
            Ok(()) => {
                self.rx
                    .basic_ack(msg.delivery_tag(), self.ack_opts.clone())
                    .await
                    .map_err(crate::Error::from)?;
                Ok(msg.data().to_vec())
            }
            Err(crate::MessageError::Drop) => Ok(vec![]),
            Err(crate::MessageError::Reject) => {
                self.rx
                    .basic_reject(msg.delivery_tag(), self.rej_opts.clone())
                    .await
                    .map_err(crate::Error::from)?;
                Ok(vec![])
            }
            Err(crate::MessageError::Nack) => {
                self.rx
                    .basic_nack(msg.delivery_tag(), self.nack_opts.clone())
                    .await
                    .map_err(crate::Error::from)?;
                Ok(vec![])
            }
        }
    }
}
