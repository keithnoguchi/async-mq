// SPDX-License-Identifier: APACHE-2.0 AND MIT
//! Zero-cost [lapin] abstraction crate
//!
//! [lapin]: https://crates.io/crates/lapin
pub use client::{Client, Connection};
pub use consume::{Consumer, ConsumerBuilder, ConsumerExt};
pub use produce::{Producer, ProducerBuilder, ProducerExt};

pub mod client;
pub mod consume;
pub mod produce;

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
    pub use crate::{Consumer, ConsumerBuilder, ConsumerExt};
    #[doc(no_inline)]
    pub use crate::{Producer, ProducerBuilder, ProducerExt};
}
