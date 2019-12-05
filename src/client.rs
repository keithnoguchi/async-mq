// SPDX-License-Identifier: GPL-2.0
use crate::Consumer;
use lapin::{Connection, ConnectionProperties, Result};

#[derive(Clone)]
pub struct Client {
    pub c: Connection,
}

impl Client {
    pub async fn new(uri: &str) -> Result<Self> {
        let c = Connection::connect(uri, ConnectionProperties::default()).await?;
        Ok(Self { c })
    }
    pub async fn consumer(&mut self) -> Result<Consumer> {
        Consumer::new(self.clone()).await
    }
}
