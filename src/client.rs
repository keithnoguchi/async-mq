// SPDX-License-Identifier: APACHE-2.0 AND MIT
//! `Client` and `Connection` structs
use std::default::Default;

/// A [non-consuming] [Connection] builder.
///
/// [Connection]: struct.Connection.html
/// [non-consuming]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html#non-consuming-builders-(preferred):
pub struct Client {
    props: lapin::ConnectionProperties,
}

impl Client {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub async fn connect(&self, uri: &str) -> crate::Result<Connection> {
        let c = lapin::Connection::connect(uri, self.props.clone())
            .await
            .map_err(crate::Error::from)?;
        Ok(Connection(c))
    }
}

impl Default for Client {
    fn default() -> Self {
        Self {
            props: lapin::ConnectionProperties::default(),
        }
    }
}

/// A [non-consuming] [ProducerBuilder] and [ConsumerBuilder] builder.
///
/// [ProducerBuilder]: ../produce/struct.ProducerBuilder.html
/// [ConsumerBuilder]: ../consume/struct.ConsumerBuilder.html
/// [non-consuming]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html#non-consuming-builders-(preferred):
#[derive(Clone)]
pub struct Connection(lapin::Connection);

#[derive(Clone)]
pub struct QueueOptions {
    pub kind: lapin::ExchangeKind,
    pub ex_opts: lapin::options::ExchangeDeclareOptions,
    pub ex_field: lapin::types::FieldTable,
    pub queue_opts: lapin::options::QueueDeclareOptions,
    pub queue_field: lapin::types::FieldTable,
    pub bind_opts: lapin::options::QueueBindOptions,
    pub bind_field: lapin::types::FieldTable,
}

impl Connection {
    /// Build a [non-consuming] [ProducerBuilder].
    ///
    /// [ProducerBuilder]: ../consume/struct.ProducerBuilder.html
    /// [non-consuming]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html#non-consuming-builders-(preferred):
    pub fn producer_builder(&self) -> crate::ProducerBuilder {
        crate::ProducerBuilder::new(self.clone())
    }
    /// Build a [non-consuming] [ConsumerBuilder].
    ///
    /// [ConsumerBuilder]: ../consume/struct.ConsumerBuilder.html
    /// [non-consuming]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html#non-consuming-builders-(preferred):
    pub fn consumer_builder(&self) -> crate::ConsumerBuilder {
        crate::ConsumerBuilder::new(self.clone())
    }
    /// channel creates a channel over the [Connection]
    /// and returns the `Future<Output = <lapin::Channel>>`.
    pub async fn channel(&self) -> crate::Result<lapin::Channel> {
        self.0.create_channel().await.map_err(crate::Error::from)
    }
    /// queue creates a channel and a queue over the [Connection]
    /// and returns the `Future<Output = <lapin::Channel, lapin::Queue>>`.
    pub async fn queue(
        &self,
        ex: &str,
        queue: &str,
        opts: QueueOptions,
    ) -> crate::Result<(lapin::Channel, lapin::Queue)> {
        let ch = self.0.create_channel().await.map_err(crate::Error::from)?;
        let q = ch
            .queue_declare(queue, opts.queue_opts, opts.queue_field)
            .await
            .map_err(crate::Error::from)?;
        if Self::is_default_exchange(ex) {
            // We don't need to bind to the exchange in case of the default
            // exchange.
            return Ok((ch, q));
        }
        ch.exchange_declare(ex, opts.kind, opts.ex_opts, opts.ex_field)
            .await
            .map_err(crate::Error::from)?;
        let routing_key = if Self::is_ephemeral_queue(queue) {
            q.name().as_str()
        } else {
            queue
        };
        ch.queue_bind(
            queue,
            ex,
            routing_key,
            opts.bind_opts.clone(),
            opts.bind_field.clone(),
        )
        .await
        .map_err(crate::Error::from)?;
        Ok((ch, q))
    }
    fn is_default_exchange(name: &str) -> bool {
        name == crate::DEFAULT_EXCHANGE
    }
    fn is_ephemeral_queue(name: &str) -> bool {
        name == crate::EPHEMERAL_QUEUE
    }
}
