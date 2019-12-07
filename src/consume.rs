// SPDX-License-Identifier: GPL-2.0
use crate::{msg, Connection};
use futures_util::stream::StreamExt;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Result};
use std::default::Default;

#[derive(Clone)]
pub struct ConsumerBuilder {
    conn: Connection,
    ex: String,
    queue: String,
    queue_options: QueueDeclareOptions,
    field_table: FieldTable,
}

impl ConsumerBuilder {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn,
            ex: String::from(""),
            queue: String::from(""),
            queue_options: QueueDeclareOptions::default(),
            field_table: FieldTable::default(),
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
    pub async fn build(&self) -> Result<Consumer> {
        let (ch, q) = match self
            .conn
            .channel(
                &self.queue,
                self.queue_options.clone(),
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
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
        {
            Ok(recv) => recv,
            Err(err) => return Err(err),
        };
        Ok(Consumer { ch, recv })
    }
}

pub struct Consumer {
    ch: lapin::Channel,
    recv: lapin::Consumer,
}

impl Consumer {
    pub async fn run(&mut self) -> Result<()> {
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
                        .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
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
    async fn send(&mut self, queue: &str, msg: &[u8]) -> Result<()> {
        self.ch
            .basic_publish(
                "",
                &queue,
                BasicPublishOptions::default(),
                msg.to_vec(),
                BasicProperties::default(),
            )
            .await?;
        Ok(())
    }
}
