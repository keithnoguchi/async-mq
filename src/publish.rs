// SPDX-License-Identifier: APACHE-2.0 AND MIT
//! produce module for Publisher and PublisherBuilder.
use crate::msg;
use futures_util::stream::StreamExt;
use lapin;

/// PublisherBuilder builds the Publisher.
#[derive(Clone)]
pub struct PublisherBuilder {
    conn: crate::Connection,
    ex: String,
    queue: String,
    queue_options: lapin::options::QueueDeclareOptions,
    field_table: lapin::types::FieldTable,
    properties: lapin::BasicProperties,
    publish_options: lapin::options::BasicPublishOptions,
}

impl PublisherBuilder {
    pub fn new(conn: crate::Connection) -> Self {
        Self {
            conn,
            ex: String::from(""),
            queue: String::from(""),
            properties: lapin::BasicProperties::default(),
            publish_options: lapin::options::BasicPublishOptions::default(),
            queue_options: lapin::options::QueueDeclareOptions::default(),
            field_table: lapin::types::FieldTable::default(),
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
    pub async fn build(&self) -> lapin::Result<Publisher> {
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
        let rx_opts = lapin::options::QueueDeclareOptions {
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
                lapin::options::BasicConsumeOptions::default(),
                lapin::types::FieldTable::default(),
            )
            .await
        {
            Ok(recv) => recv,
            Err(err) => return Err(err),
        };
        Ok(Publisher {
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

pub struct Publisher {
    tx: lapin::Channel,
    rx: lapin::Channel,
    recv: lapin::Consumer,
    ex: String,
    queue: String,
    properties: lapin::BasicProperties,
    rx_props: lapin::BasicProperties,
    publish_options: lapin::options::BasicPublishOptions,
}

impl Publisher {
    pub async fn rpc(&mut self, msg: Vec<u8>) -> lapin::Result<()> {
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
                        .basic_ack(
                            delivery.delivery_tag,
                            lapin::options::BasicAckOptions::default(),
                        )
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
    pub async fn publish(&mut self, msg: Vec<u8>) -> lapin::Result<()> {
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
