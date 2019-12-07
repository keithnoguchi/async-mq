// SPDX-License-Identifier: GPL-2.0
//! produce module for Producer and ProducerBuilder.
use crate::{msg, Connection};
use futures_util::stream::StreamExt;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Result};

/// ProducerBuilder builds the Producer.
#[derive(Clone)]
pub struct ProducerBuilder {
    conn: Connection,
    ex: String,
    queue: String,
    queue_options: QueueDeclareOptions,
    field_table: FieldTable,
    properties: BasicProperties,
    publish_options: BasicPublishOptions,
}

impl ProducerBuilder {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn,
            ex: String::from(""),
            queue: String::from(""),
            properties: BasicProperties::default(),
            publish_options: BasicPublishOptions::default(),
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
    pub async fn build(&self) -> Result<Producer> {
        let tx = match self
            .conn
            .channel(
                &self.queue,
                self.queue_options.clone(),
                self.field_table.clone(),
            )
            .await
        {
            Ok((ch, _)) => ch,
            Err(err) => return Err(err),
        };
        let rx_opts = QueueDeclareOptions {
            exclusive: true,
            auto_delete: true,
            ..self.queue_options.clone()
        };
        let (rx, q) = match self
            .conn
            .channel("", rx_opts, self.field_table.clone())
            .await
        {
            Ok((ch, q)) => (ch, q),
            Err(err) => return Err(err),
        };
        let recv = match rx
            .basic_consume(
                &q,
                "producer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
        {
            Ok(recv) => recv,
            Err(err) => return Err(err),
        };
        Ok(Producer {
            tx,
            rx,
            recv,
            ex: self.ex.clone(),
            queue: self.queue.clone(),
            properties: self.properties.clone(),
            rx_props: self.properties.clone().with_reply_to(q.name().clone()),
            publish_options: self.publish_options.clone(),
        })
    }
}

pub struct Producer {
    tx: lapin::Channel,
    rx: lapin::Channel,
    recv: lapin::Consumer,
    ex: String,
    queue: String,
    properties: BasicProperties,
    rx_props: BasicProperties,
    publish_options: BasicPublishOptions,
}

impl Producer {
    pub async fn rpc(&mut self, msg: Vec<u8>) -> Result<()> {
        self.tx
            .basic_publish(
                &self.ex,
                &self.queue,
                self.publish_options.clone(),
                msg,
                self.rx_props.clone(),
            )
            .await?;
        if let Some(delivery) = self.recv.next().await {
            match delivery {
                Ok(delivery) => {
                    let msg = msg::get_root_as_message(&delivery.data);
                    eprint!("{}", msg.msg().unwrap());
                    if let Err(err) = self
                        .rx
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
    pub async fn publish(&mut self, msg: Vec<u8>) -> Result<()> {
        self.tx
            .basic_publish(
                &self.ex,
                &self.queue,
                self.publish_options.clone(),
                msg,
                self.properties.clone(),
            )
            .await
    }
}
