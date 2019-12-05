// SPDX-License-Identifier: GPL-2.0
use crate::Client;
use lapin::options::{BasicPublishOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Channel, Result};
use std::default::Default;

pub struct Producer {
    pub exchange: String,
    pub queue: String,
    pub properties: BasicProperties,
    pub publish_options: BasicPublishOptions,
    pub queue_options: QueueDeclareOptions,
    client: Option<Client>,
    channel: Option<Channel>,
}

impl Producer {
    pub fn new(c: Client, queue: String) -> Self {
        Self {
            client: Some(c),
            queue,
            ..Default::default()
        }
    }
    pub async fn publish(&mut self, msg: Vec<u8>) -> Result<()> {
        let ch = match &self.channel {
            Some(ch) => ch,
            None => {
                if let Err(err) = self.create_channel().await {
                    return Err(err);
                }
                self.channel.as_ref().unwrap()
            }
        };
        ch.basic_publish(
            &self.exchange,
            &self.queue,
            self.publish_options.clone(),
            msg,
            self.properties.clone(),
        )
        .await
    }
    async fn create_channel(&mut self) -> Result<()> {
        let ch = match self.client.as_ref().unwrap().0.create_channel().await {
            Ok(ch) => ch,
            Err(err) => return Err(err),
        };
        if let Err(err) = ch
            .queue_declare(
                &self.queue,
                self.queue_options.clone(),
                FieldTable::default(),
            )
            .await
        {
            return Err(err);
        }
        self.channel = Some(ch);
        Ok(())
    }
}

impl Default for Producer {
    fn default() -> Self {
        Self {
            exchange: String::from(""),
            queue: String::from("/"),
            properties: BasicProperties::default(),
            publish_options: BasicPublishOptions::default(),
            queue_options: QueueDeclareOptions::default(),
            client: None,
            channel: None,
        }
    }
}
