// SPDX-License-Identifier: APACHE-2.0 AND MIT
//! [Client] and [Connection] structs
//!
//! [Client]: struct.Client.html
//! [Connection]: struct.Connection.html
use lapin::options::QueueDeclareOptions;
use lapin::types::FieldTable;
use lapin::{Channel, ConnectionProperties, Queue, Result};
use std::default::Default;

/// A non-consuming [Connection] builder.
///
/// [Connection]: struct.Connection.html
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

/// A non-consuming [ProducerBuilder] and [ConsumerBuilder] builder.
///
/// [ProducerBuilder]: ../produce/struct.ProducerBuilder.html
/// [ConsumerBuilder]: ../consume/struct.ConsumerBuilder.html
#[derive(Clone)]
pub struct Connection(lapin::Connection);

impl Connection {
    /// channel creates a channel and a queue over the [Connection]
    /// and returns the `Future<Output = <lapin::Channel, lapin::Queue>>`.
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
