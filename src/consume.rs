// SPDX-License-Identifier: GPL-2.0
use crate::Client;
use lapin::options::{BasicConsumeOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use lapin::{Channel, Result};
use std::default::Default;

pub struct Consumer {
    pub channel: Channel,
    pub consumer: lapin::Consumer,
}

pub struct ConsumerBuilder {
    pub queue_options: QueueDeclareOptions,
    client: Option<Client>,
}

impl ConsumerBuilder {
    pub fn new(c: Client) -> Self {
        Self {
            client: Some(c),
            ..Default::default()
        }
    }
    pub async fn consumer(&mut self, queue: &str) -> Result<Consumer> {
        let channel = match self.client.as_ref().unwrap().0.create_channel().await {
            Ok(ch) => ch,
            Err(err) => return Err(err),
        };
        let q = match channel
            .queue_declare(queue, self.queue_options.clone(), FieldTable::default())
            .await
        {
            Ok(q) => q,
            Err(err) => return Err(err),
        };
        let consumer = match channel
            .clone()
            .basic_consume(
                &q,
                "my_consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
        {
            Ok(c) => c,
            Err(err) => return Err(err),
        };
        Ok(Consumer { channel, consumer })
    }
}

impl Default for ConsumerBuilder {
    fn default() -> Self {
        Self {
            queue_options: QueueDeclareOptions::default(),
            client: None,
        }
    }
}
