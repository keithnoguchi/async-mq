// SPDX-License-Identifier: APACHE-2.0 AND MIT
use crate::msg;
use futures_util::stream::StreamExt;
use lapin;

#[derive(Clone)]
pub struct SubscriberBuilder {
    conn: crate::Connection,
    ex: String,
    queue: String,
    queue_opts: lapin::options::QueueDeclareOptions,
    field_table: lapin::types::FieldTable,
    tx_props: lapin::BasicProperties,
    tx_opts: lapin::options::BasicPublishOptions,
    rx_opts: lapin::options::BasicConsumeOptions,
    ack_opts: lapin::options::BasicAckOptions,
    consumer: Box<dyn crate::Consumer + Send>,
}

impl SubscriberBuilder {
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
            consumer: Box::new(crate::consume::EchoConsumer {}),
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
    pub fn with_consumer(&mut self, consumer: Box<dyn crate::Consumer + Send>) -> &mut Self {
        self.consumer = consumer;
        self
    }
    pub async fn build(&self) -> lapin::Result<Subscriber> {
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
        Ok(Subscriber {
            ch,
            consume,
            tx_props: self.tx_props.clone(),
            tx_opts: self.tx_opts.clone(),
            ack_opts: self.ack_opts.clone(),
            consumer: self.consumer.clone(),
        })
    }
}

pub struct Subscriber {
    ch: lapin::Channel,
    consume: lapin::Consumer,
    tx_props: lapin::BasicProperties,
    tx_opts: lapin::options::BasicPublishOptions,
    ack_opts: lapin::options::BasicAckOptions,
    consumer: Box<dyn crate::Consumer + Send>,
}

impl Subscriber {
    pub fn with_consumer(&mut self, consumer: Box<dyn crate::Consumer + Send>) -> &mut Self {
        self.consumer = consumer;
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
    async fn recv(&mut self, msg: lapin::message::Delivery) -> lapin::Result<()> {
        let delivery_tag = msg.delivery_tag;
        let reply_to = msg.properties.reply_to();
        match self.consumer.consume(msg.data).await {
            Err(err) => return Err(err),
            Ok(msg) => {
                if let Some(reply_to) = reply_to {
                    self.send(reply_to.as_str(), &msg).await?;
                } else {
                    let msg = msg::get_root_as_message(&msg);
                    print!("{}", msg.msg().unwrap());
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
