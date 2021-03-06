// SPDX-License-Identifier: Apache-2.0 AND MIT
//! Zero-cost async-await abstraction of [lapin] [AMQP] client crate
//!
//! [lapin]: https://crates.io/crates/lapin
//! [amqp]: https://www.amqp.org
pub use client::{Client, Connection};
pub use consume::{Consumer, ConsumerBuilder};
pub use error::Error;
pub use message::{Message, MessageError, MessagePeek, MessageProcess};
pub use produce::{Producer, ProducerBuilder};

pub mod client;
pub mod consume;
pub mod error;
pub mod message;
pub mod produce;

/// Crate local type aliases for less typing.  Those are meant for the
/// internal use cases and won't be published.
type Result<T> = std::result::Result<T, error::Error>;

/// DEFAULT_EXCHANGE and DEFAULT_QUEUE for both consumers and producers.
const DEFAULT_EXCHANGE: &str = "";
const DEFAULT_QUEUE: &str = "";
const EPHEMERAL_QUEUE: &str = "";

/// A "prelude" for the crate
///
/// This prelude is similar to the standard library's prelude in that you'll
/// almost always want to import its entire contents, but unlike the standard
/// library's prelude you'll have to do so manually.
///
/// We may add items to this over time as they become ubiquitous as well.
pub mod prelude {
    #[doc(no_inline)]
    pub use crate::{Client, Connection};
    #[doc(no_inline)]
    pub use crate::{Consumer, ConsumerBuilder};
    #[doc(no_inline)]
    pub use crate::{Producer, ProducerBuilder};
}
