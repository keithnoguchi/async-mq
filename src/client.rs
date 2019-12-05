// SPDX-License-Identifier: GPL-2.0
use lapin::{Connection, ConnectionProperties, Result};

#[derive(Clone)]
pub struct Client(pub Connection);

impl Client {
    pub async fn new(uri: &str) -> Result<Self> {
        let c = Connection::connect(uri, ConnectionProperties::default()).await?;
        Ok(Self(c))
    }
}
