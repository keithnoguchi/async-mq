// SPDX-License-Identifier: GPL-2.0
use crate::Client;
use lapin::options::{BasicPublishOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Channel, Result};

pub struct Producer {
    queue: &'static str,
    channel: Channel,
}

impl Producer {
    pub async fn new(c: Client, queue: &'static str) -> Result<Self> {
        let channel = c.c.create_channel().await?;
        channel
            .queue_declare(queue, QueueDeclareOptions::default(), FieldTable::default())
            .await?;
        Ok(Self { channel, queue })
    }
    pub async fn publish(&mut self, msg: Vec<u8>) -> Result<()> {
        self.channel
            .basic_publish(
                "",
                self.queue,
                BasicPublishOptions::default(),
                msg,
                BasicProperties::default(),
            )
            .await
    }
}
