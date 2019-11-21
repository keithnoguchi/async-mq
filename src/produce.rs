// SPDX-License-Identifier: GPL-2.0
use lapin::{options::*, types::FieldTable};
use lapin::{Channel, Connection, ConnectionProperties, Result};

pub struct Producer {
    pub channel: Channel,
}

impl Producer {
    pub async fn new(uri: &str, name: &str) -> Result<Producer> {
        let c = Connection::connect(uri, ConnectionProperties::default()).await?;
        let channel = c.create_channel().await?;
        channel
            .queue_declare(name, QueueDeclareOptions::default(), FieldTable::default())
            .await?;
        Ok(Producer { channel })
    }
}
