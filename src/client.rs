// SPDX-License-Identifier: GPL-2.0
use lapin::options::QueueDeclareOptions;
use lapin::types::FieldTable;
use lapin::{Channel, Connection, ConnectionProperties, Queue, Result};

/// ClientBuilder builds the Client.
pub struct ClientBuilder {
    uri: String,
    props: ConnectionProperties,
}

impl ClientBuilder {
    pub fn new(uri: String) -> Self {
        Self {
            uri,
            props: ConnectionProperties::default(),
        }
    }
    pub async fn build(&self) -> Result<Client> {
        let c = match Connection::connect(&self.uri, self.props.clone()).await {
            Ok(c) => c,
            Err(err) => return Err(err),
        };
        Ok(Client(c))
    }
}

/// Client represents the connection to the AMQP broker.
#[derive(Clone)]
pub struct Client(Connection);

impl Client {
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
