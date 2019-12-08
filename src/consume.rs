// SPDX-License-Identifier: APACHE-2.0 AND MIT
//! [ConsumerBuilder], [Consumer] structs, and [ConsumerExt] trait
//!
//! [ConsumerBuilder]: struct.ConsumerBuilder.html
//! [Consumer]: struct.Consumer.html
//! [ConsumerExt]: trait.ConsumerExt.html
use async_trait::async_trait;
use futures_util::stream::StreamExt;
use lapin;

/// A [Consumer] builder.
///
/// [Consumer]: struct.Consumer.html
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
    extension: Box<dyn crate::ConsumerExt + Send>,
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
            extension: Box::new(EchoMessager {}),
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
    /// Override the default [EchoMessenger] [ConsumerExt] trait object.
    ///
    /// [EchoMessenger]: struct.EchoMessenger.html
    /// [ConsumerExt]: trait.ConsumerExt.html
    pub fn with_ext(&mut self, extension: Box<dyn crate::ConsumerExt + Send>) -> &mut Self {
        self.extension = extension;
        self
    }
    pub async fn build(&self) -> lapin::Result<Consumer> {
        let (ch, q) = match self
            .conn
            .channel(
                &self.queue,
                self.queue_opts.clone(),
                self.field_table.clone(),
            )
            .await
        {
            Ok((ch, q)) => (ch, q),
            Err(err) => return Err(err),
        };
        let consume = match ch
            .clone()
            .basic_consume(
                &q,
                "consumer",
                self.rx_opts.clone(),
                self.field_table.clone(),
            )
            .await
        {
            Ok(c) => c,
            Err(err) => return Err(err),
        };
        Ok(Consumer {
            ch,
            consume,
            tx_props: self.tx_props.clone(),
            tx_opts: self.tx_opts.clone(),
            ack_opts: self.ack_opts.clone(),
            extension: self.extension.clone(),
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
    extension: Box<dyn crate::ConsumerExt + Send>,
}

impl Consumer {
    /// Override the default [EchoMessenger] [ConsumerExt] trait object.
    ///
    /// [EchoMessenger]: struct.EchoMessenger.html
    /// [ConsumerExt]: trait.ConsumerExt.html
    pub fn with_ext(&mut self, extension: Box<dyn crate::ConsumerExt + Send>) -> &mut Self {
        self.extension = extension;
        self
    }
    pub async fn run(&mut self) -> lapin::Result<()> {
        while let Some(msg) = self.consume.next().await {
            match msg {
                Ok(msg) => self.recv(msg).await?,
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }
    /// Transfer the received message to [ConsumerExt] and acknowledge
    /// it to the broker, or nack in case [ConsumerExt] implementor returns
    /// error.  In case of the message contains `reply_to` field, it will
    /// send back to the response queue specified in `reply_to` field in
    /// the message.
    ///
    /// [ConsumerExt]: trait.ConsumerExt.html
    async fn recv(&mut self, msg: lapin::message::Delivery) -> lapin::Result<()> {
        let delivery_tag = msg.delivery_tag;
        let reply_to = msg.properties.reply_to();
        match self.extension.recv(msg.data).await {
            Err(err) => return Err(err),
            Ok(msg) => {
                if let Some(reply_to) = reply_to {
                    self.send(reply_to.as_str(), &msg).await?;
                } else {
                    eprint!("{:?}", msg);
                }
                if let Err(err) = self.ch.basic_ack(delivery_tag, self.ack_opts.clone()).await {
                    return Err(err);
                }
            }
        }
        Ok(())
    }
    async fn send(&mut self, queue: &str, msg: &[u8]) -> lapin::Result<()> {
        self.ch
            .basic_publish(
                "",
                &queue,
                self.tx_opts.clone(),
                msg.to_vec(),
                self.tx_props.clone(),
            )
            .await?;
        Ok(())
    }
}

/// A trait to extend the [Consumer] capability.
///
/// [Consumer]: struct.Consumer.html
#[async_trait]
pub trait ConsumerExt {
    /// Async method to transfer the message to [ConsumerExt] implementor.
    ///
    /// [ConsumerExt]: trait.ConsumerExt.html
    async fn recv(&mut self, msg: Vec<u8>) -> lapin::Result<Vec<u8>>;
    fn box_clone(&self) -> Box<dyn ConsumerExt + Send>;
}

// https://users.rust-lang.org/t/solved-is-it-possible-to-clone-a-boxed-trait-object/1714/6
impl Clone for Box<dyn ConsumerExt + Send> {
    fn clone(&self) -> Box<dyn ConsumerExt + Send> {
        self.box_clone()
    }
}

/// A default [ConsumerExt] implementor that echoes back the received message.
///
/// [ConsumerExt]: trait.ConsumerExt.html
#[derive(Clone)]
pub struct EchoMessager;

#[async_trait]
impl ConsumerExt for EchoMessager {
    /// Echoe back the received message.
    async fn recv(&mut self, msg: Vec<u8>) -> lapin::Result<Vec<u8>> {
        Ok(msg)
    }
    fn box_clone(&self) -> Box<dyn ConsumerExt + Send> {
        Box::new((*self).clone())
    }
}
