// SPDX-License-Identifier: APACHE-2.0 AND MIT
//! client module for the connection to the message queue broker.
use lapin::options::QueueDeclareOptions;
use lapin::types::FieldTable;
use lapin::{Channel, ConnectionProperties, Queue, Result};
use std::default::Default;

/// Client is the non-consuming builder for the Connection.
pub struct Client {
    props: ConnectionProperties,
}

impl Client {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub async fn connect(&self, uri: &str) -> Result<Connection> {
        let c = match lapin::Connection::connect(uri, self.props.clone()).await {
            Ok(c) => c,
            Err(err) => return Err(err),
        };
        Ok(Connection(c))
    }
}

impl Default for Client {
    fn default() -> Self {
        Self {
            props: ConnectionProperties::default(),
        }
    }
}

/// Connection represents the connection to the message queue broker.
#[derive(Clone)]
pub struct Connection(lapin::Connection);

impl Connection {
    /// channel creates the channel and the queue over the connection
    /// and returns the Future<Output = <Channel, Queue>>.
    pub async fn channel(
        &self,
        queue: &str,
        opts: QueueDeclareOptions,
        field: FieldTable,
    ) -> Result<(Channel, Queue)> {
        let ch = match self.0.create_channel().await {
            Ok(ch) => ch,
            Err(err) => return Err(err),
        };
        let q = match ch.queue_declare(queue, opts, field).await {
            Ok(q) => q,
            Err(err) => return Err(err),
        };
        Ok((ch, q))
    }
}
