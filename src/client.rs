// SPDX-License-Identifier: GPL-2.0
use lapin::options::QueueDeclareOptions;
use lapin::types::FieldTable;
use lapin::{Channel, Connection, ConnectionProperties, Queue, Result};
use std::default::Default;

#[derive(Clone)]
pub struct Client {
    pub properties: ConnectionProperties,
    uri: String,
    connection: Option<Connection>,
}

impl Client {
    pub fn new(uri: String) -> Self {
        Self {
            uri,
            ..Default::default()
        }
    }
    pub async fn connect(&mut self) -> Result<()> {
        let c = match Connection::connect(&self.uri, self.properties.clone()).await {
            Ok(c) => c,
            Err(err) => return Err(err),
        };
        self.connection = Some(c);
        Ok(())
    }
    pub async fn channel_and_queue(
        &self,
        queue: &str,
        options: QueueDeclareOptions,
        table: FieldTable,
    ) -> Result<(Channel, Queue)> {
        let ch = match self.connection.as_ref().unwrap().create_channel().await {
            Ok(ch) => ch,
            Err(err) => return Err(err),
        };
        let q = match ch.queue_declare(queue, options, table).await {
            Ok(q) => q,
            Err(err) => return Err(err),
        };
        Ok((ch, q))
    }
}

impl Default for Client {
    fn default() -> Self {
        Self {
            uri: String::from(""),
            properties: ConnectionProperties::default(),
            connection: None,
        }
    }
}
