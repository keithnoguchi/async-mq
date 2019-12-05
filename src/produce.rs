// SPDX-License-Identifier: GPL-2.0
use crate::{msg, Client};
use futures_util::stream::StreamExt;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Channel, Result};
use std::default::Default;

pub struct Producer {
    pub exchange: String,
    pub queue: String,
    pub properties: BasicProperties,
    pub publish_options: BasicPublishOptions,
    pub queue_options: QueueDeclareOptions,
    pub field_table: FieldTable,
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
    pub async fn rpc(&mut self, msg: Vec<u8>) -> Result<()> {
        let ch = match &self.channel {
            Some(ch) => ch,
            None => {
                if let Err(err) = self.create_channel().await {
                    return Err(err);
                }
                self.channel.as_ref().unwrap()
            }
        };
        let opts = QueueDeclareOptions {
            exclusive: true,
            auto_delete: true,
            ..self.queue_options.clone()
        };
        let (reply_ch, q) = match self
            .client
            .as_ref()
            .unwrap()
            .channel_and_queue("", opts, self.field_table.clone())
            .await
        {
            Ok((ch, q)) => (ch, q),
            Err(err) => return Err(err),
        };
        ch.basic_publish(
            &self.exchange,
            &self.queue,
            self.publish_options.clone(),
            msg,
            self.properties.clone().with_reply_to(q.name().clone()),
        )
        .await?;
        let mut consumer = match reply_ch
            .basic_consume(
                &q,
                "producer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
        {
            Ok(c) => c,
            Err(err) => return Err(err),
        };
        if let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(delivery) => {
                    let msg = msg::get_root_as_message(&delivery.data);
                    eprint!("{}", msg.msg().unwrap());
                    if let Err(err) = reply_ch
                        .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                        .await
                    {
                        return Err(err);
                    }
                }
                Err(err) => return Err(err),
            }
        }
        if let Err(err) = reply_ch.close(0, "closing").await {
            return Err(err);
        }
        Ok(())
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
        let ch = match self
            .client
            .as_ref()
            .unwrap()
            .channel_and_queue(
                &self.queue,
                self.queue_options.clone(),
                self.field_table.clone(),
            )
            .await
        {
            Ok((ch, _)) => ch,
            Err(err) => return Err(err),
        };
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
            field_table: FieldTable::default(),
            client: None,
            channel: None,
        }
    }
}
