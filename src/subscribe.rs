// SPDX-License-Identifier: GPL-2.0
use crate::msg;
use futures::future::BoxFuture;
use futures_util::{future::FutureExt, stream::StreamExt};

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
        }
    }
    pub fn exchange(&mut self, exchange: String) -> &Self {
        self.ex = exchange;
        self
    }
    pub fn queue(&mut self, queue: String) -> &Self {
        self.queue = queue;
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
        let recv = match ch
            .clone()
            .basic_consume(
                &q,
                "consumer",
                self.rx_opts.clone(),
                self.field_table.clone(),
            )
            .await
        {
            Ok(recv) => recv,
            Err(err) => return Err(err),
        };
        Ok(Subscriber {
            ch,
            recv,
            tx_props: self.tx_props.clone(),
            tx_opts: self.tx_opts.clone(),
            ack_opts: self.ack_opts.clone(),
        })
    }
}

pub struct Subscriber {
    ch: lapin::Channel,
    recv: lapin::Consumer,
    tx_props: lapin::BasicProperties,
    tx_opts: lapin::options::BasicPublishOptions,
    ack_opts: lapin::options::BasicAckOptions,
}

impl Subscriber {
    pub async fn run(&mut self) -> lapin::Result<()> {
        while let Some(delivery) = self.recv.next().await {
            match delivery {
                Ok(delivery) => {
                    if let Some(reply_to) = delivery.properties.reply_to() {
                        self.send(reply_to.as_str(), &delivery.data).await?;
                    } else {
                        let msg = msg::get_root_as_message(&delivery.data);
                        print!("{}", msg.msg().unwrap());
                    }
                    if let Err(err) = self
                        .ch
                        .basic_ack(delivery.delivery_tag, self.ack_opts.clone())
                        .await
                    {
                        return Err(err);
                    }
                }
                Err(err) => return Err(err),
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

pub trait Consumer<'future> {
    fn consume(msg: Vec<u8>) -> BoxFuture<'future, lapin::Result<()>>;
}

#[allow(dead_code)]
struct Noop;

impl<'future> Consumer<'future> for Noop {
    fn consume(_msg: Vec<u8>) -> BoxFuture<'future, lapin::Result<()>> {
        async { Ok(()) }.boxed()
    }
}
