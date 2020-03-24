// SPDX-License-Identifier: Apache-2.0 AND MIT
//! `ConsumerBuilder` and `Consumer` structs
use futures::stream::{Stream, StreamExt};
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
    processor: Box<dyn crate::MessageProcess + Send + Sync>,
}

impl ConsumerBuilder {
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
            processor: Box::new(crate::message::EchoProcessor {}),
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
    /// Use the provided [MessageProcess] trait object.
    ///
    /// [MessageProcess]: ../message/trait.MessageProcess.html
    pub fn with_processor(
        &mut self,
        processor: Box<dyn crate::MessageProcess + Send + Sync>,
    ) -> &mut Self {
        self.processor = processor;
        self
    }
    pub async fn build(&self) -> crate::Result<Consumer> {
        let opts = crate::client::QueueOptions {
            kind: self.kind.clone(),
            ex_opts: self.ex_opts.clone(),
            ex_field: self.field_table.clone(),
            queue_opts: self.queue_opts.clone(),
            queue_field: self.field_table.clone(),
            bind_opts: self.bind_opts.clone(),
            bind_field: self.field_table.clone(),
        };
        let (ch, q) = self.conn.queue(&self.ex, &self.queue, opts).await?;
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
            ex: self.ex.clone(),
            tx_props: self.tx_props.clone(),
            tx_opts: self.tx_opts.clone(),
            ack_opts: self.ack_opts.clone(),
            rej_opts: self.rej_opts.clone(),
            processor: self.processor.clone(),
        })
    }
}

/// A zero-cost [lapin::Consumer] abstruction type.
///
/// [lapin::Consumer]: https://docs.rs/lapin/latest/lapin/struct.Consumer.html
pub struct Consumer {
    ch: lapin::Channel,
    consume: lapin::Consumer,
    ex: String,
    tx_props: lapin::BasicProperties,
    tx_opts: lapin::options::BasicPublishOptions,
    ack_opts: lapin::options::BasicAckOptions,
    rej_opts: lapin::options::BasicRejectOptions,
    processor: Box<dyn crate::MessageProcess + Send + Sync>,
}

impl Consumer {
    /// Use the provided [MessageProcess] trait object.
    ///
    /// [MessageProcess]: ../message/trait.MessageProcess.html
    pub fn with_processor(
        &mut self,
        processor: Box<dyn crate::MessageProcess + Send + Sync>,
    ) -> &mut Self {
        self.processor = processor;
        self
    }
    pub async fn run(&mut self) -> crate::Result<()> {
        while let Some(msg) = self.consume.next().await {
            match msg {
                Ok(msg) => {
                    let req = &crate::Message::new(msg);
                    match self.processor.process(req).await {
                        Ok(resp) => self.response(req, &resp).await?,
                        Err(_err) => self.reject(req).await?,
                    }
                }
                Err(err) => return Err(crate::Error::from(err)),
            }
        }
        Ok(())
    }
    pub async fn response(&mut self, req: &crate::Message, resp: &[u8]) -> crate::Result<()> {
        if let Some(reply_to) = req.reply_to() {
            self.send(reply_to, resp).await?;
        }
        self.ch
            .basic_ack(req.delivery_tag(), self.ack_opts.clone())
            .await
            .map_err(crate::Error::from)?;
        Ok(())
    }
    pub async fn reject(&mut self, req: &crate::Message) -> crate::Result<()> {
        self.ch
            .basic_reject(req.delivery_tag(), self.rej_opts.clone())
            .await
            .map_err(crate::Error::from)?;
        Ok(())
    }
    async fn send(&mut self, routing_key: &str, msg: &[u8]) -> crate::Result<()> {
        self.ch
            .basic_publish(
                &self.ex,
                &routing_key,
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
            Poll::Ready(Some(Ok(msg))) => Poll::Ready(Some(Ok(crate::Message::new(msg)))),
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(err.into()))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
