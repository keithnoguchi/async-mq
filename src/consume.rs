// SPDX-License-Identifier: GPL-2.0
use crate::Client;
use lapin::Result;
use lapin::{options::*, types::FieldTable};

pub struct Consumer {
    client: Option<Client>,
}

impl Consumer {
    pub fn new(c: Client) -> Self {
        Self { client: Some(c) }
    }
    pub async fn worker(&mut self, queue: &str) -> Result<(lapin::Consumer, lapin::Channel)> {
        let c = match self.client.as_ref().unwrap().0.create_channel().await {
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
