// SPDX-License-Identifier: GPL-2.0
use lapin::options::QueueDeclareOptions;
use lapin::types::FieldTable;
use lapin::{Channel, Connection, ConnectionProperties, Queue, Result};

#[derive(Clone)]
pub struct Client(pub Connection);

impl Client {
    pub async fn new(uri: &str) -> Result<Self> {
        let c = Connection::connect(uri, ConnectionProperties::default()).await?;
        Ok(Self(c))
    }
    pub async fn channel_and_queue(
        &self,
        queue: &str,
        options: QueueDeclareOptions,
        table: FieldTable,
    ) -> Result<(Channel, Queue)> {
        let ch = match self.0.create_channel().await {
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
