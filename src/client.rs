// SPDX-License-Identifier: GPL-2.0
use lapin::{Connection, ConnectionProperties, Result};

pub struct Client {
    pub c: Connection,
}

impl Client {
    pub async fn new(uri: &str) -> Result<Self> {
        let c = Connection::connect(uri, ConnectionProperties::default()).await?;
        Ok(Self { c })
    }
}
