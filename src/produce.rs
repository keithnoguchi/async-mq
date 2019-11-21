// SPDX-License-Identifier: GPL-2.0
use lapin::options::{BasicPublishOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Channel, Connection, ConnectionProperties, Result};

pub struct Producer {
    queue: &'static str,
    channel: Channel,
}

impl Producer {
    pub async fn new(uri: &str, queue: &'static str) -> Result<Producer> {
        let c = Connection::connect(uri, ConnectionProperties::default()).await?;
        let channel = c.create_channel().await?;
        channel
            .queue_declare(queue, QueueDeclareOptions::default(), FieldTable::default())
            .await?;
        Ok(Producer { channel, queue })
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
