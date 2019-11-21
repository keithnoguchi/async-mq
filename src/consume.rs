// SPDX-License-Identifier: GPL-2.0
use lapin::{options::*, types::FieldTable};
use lapin::{Connection, ConnectionProperties, Result};

pub struct Consumer {
    c: Connection,
}

impl Consumer {
    pub async fn new(uri: &str) -> Result<Consumer> {
        let c = Connection::connect(uri, ConnectionProperties::default()).await?;
        Ok(Consumer { c })
    }
    pub async fn worker(&mut self, queue: &str) -> Result<(lapin::Consumer, lapin::Channel)> {
        let c = match self.c.create_channel().await {
            Ok(c) => c,
            Err(err) => return Err(err),
        };
        let q = match c
            .queue_declare(queue, QueueDeclareOptions::default(), FieldTable::default())
            .await
        {
            Ok(q) => q,
            Err(err) => return Err(err),
        };
        let consumer = match c
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
        Ok((consumer, c))
    }
}
