// SPDX-License-Identifier: GPL-2.0
use lapin::{Connection, ConnectionProperties, Result};

pub struct Producer {
    pub c: Connection,
}

impl Producer {
    pub async fn new(uri: &str) -> Result<Producer> {
        let c = Connection::connect(uri, ConnectionProperties::default()).await?;
        Ok(Producer { c })
    }
}
