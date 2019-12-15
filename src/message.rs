// SPDX-License-Identifier: APACHE-2.0 AND MIT
//! [Message] struct
//!
//! [Message]: struct.Message.html
use lapin;

/// A zero-cost [lapin::message::Delivery] [newtype].
///
/// [lapin::message::Delivery]: https://docs.rs/lapin/latest/lapin/message/struct.Delivery.html
/// [newtype]: https://doc.rust-lang.org/1.0.0/style/features/types/newtype.html
pub struct Message(pub lapin::message::Delivery);

impl Message {
    pub fn data(&self) -> &[u8] {
        &self.0.data
    }
}
