// SPDX-License-Identifier: GPL-2.0
use lapin::{Connection, ConnectionProperties, Result};

pub struct Client;

impl Client {
    pub async fn connect(uri: &str) -> Result<Connection> {
        Connection::connect(uri, ConnectionProperties::default()).await
    }
}
