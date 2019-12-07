// SPDX-License-Identifier: APACHE-2.0 AND MIT
//! produce module for Publisher and PublisherBuilder.
use futures_util::stream::StreamExt;
use lapin;

/// PublisherBuilder builds the Publisher.
#[derive(Clone)]
pub struct PublisherBuilder {
    conn: crate::Connection,
    ex: String,
    queue: String,
    queue_opts: lapin::options::QueueDeclareOptions,
    field_table: lapin::types::FieldTable,
    tx_props: lapin::BasicProperties,
    tx_opts: lapin::options::BasicPublishOptions,
    rx_opts: lapin::options::BasicConsumeOptions,
    ack_opts: lapin::options::BasicAckOptions,
    producer: Box<dyn crate::Producer + Send>,
}

impl PublisherBuilder {
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
            producer: Box::new(crate::produce::FlatbufferDumpProducer {}),
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
    pub fn with_producer(&mut self, producer: Box<dyn crate::Producer + Send>) -> &mut Self {
        self.producer = producer;
        self
    }
    pub async fn build(&self) -> lapin::Result<Publisher> {
        let tx = match self
            .conn
            .channel(
                &self.queue,
                self.queue_opts.clone(),
                self.field_table.clone(),
            )
            .await
        {
            Ok((ch, _)) => ch,
            Err(err) => return Err(err),
        };
        let opts = lapin::options::QueueDeclareOptions {
            exclusive: true,
            auto_delete: true,
            ..self.queue_opts.clone()
        };
        let (rx, q) = match self.conn.channel("", opts, self.field_table.clone()).await {
            Ok((ch, q)) => (ch, q),
            Err(err) => return Err(err),
        };
        let consume = match rx
            .basic_consume(
                &q,
                "producer",
                self.rx_opts.clone(),
                self.field_table.clone(),
            )
            .await
        {
            Ok(c) => c,
            Err(err) => return Err(err),
        };
        Ok(Publisher {
            tx,
            rx,
            consume,
            ex: self.ex.clone(),
            queue: self.queue.clone(),
            tx_props: self.tx_props.clone(),
            rx_props: self.tx_props.clone().with_reply_to(q.name().clone()),
            tx_opts: self.tx_opts.clone(),
            ack_opts: self.ack_opts.clone(),
            producer: self.producer.clone(),
        })
    }
}

pub struct Publisher {
    tx: lapin::Channel,
    rx: lapin::Channel,
    consume: lapin::Consumer,
    ex: String,
    queue: String,
    tx_props: lapin::BasicProperties,
    rx_props: lapin::BasicProperties,
    tx_opts: lapin::options::BasicPublishOptions,
    ack_opts: lapin::options::BasicAckOptions,
    producer: Box<dyn crate::Producer + Send>,
}

impl Publisher {
    pub fn with_producer(&mut self, producer: Box<dyn crate::Producer + Send>) -> &mut Self {
        self.producer = producer;
        self
    }
    pub async fn rpc(&mut self, msg: Vec<u8>) -> lapin::Result<()> {
        self.tx
            .basic_publish(
                &self.ex,
                &self.queue,
                self.tx_opts.clone(),
                msg,
                self.rx_props.clone(),
            )
            .await?;
        if let Some(msg) = self.consume.next().await {
            match msg {
                Ok(msg) => self.recv(msg).await?,
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }
    pub async fn publish(&mut self, msg: Vec<u8>) -> lapin::Result<()> {
        self.tx
            .basic_publish(
                &self.ex,
                &self.queue,
                self.tx_opts.clone(),
                msg,
                self.tx_props.clone(),
            )
            .await
    }
    async fn recv(&mut self, msg: lapin::message::Delivery) -> lapin::Result<()> {
        let delivery_tag = msg.delivery_tag;
        if let Ok(()) = self.producer.recv(msg.data).await {
            if let Err(err) = self.rx.basic_ack(delivery_tag, self.ack_opts.clone()).await {
                return Err(err);
            }
        }
        Ok(())
    }
}
