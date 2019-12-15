// SPDX-License-Identifier: APACHE-2.0 AND MIT
//! [Message] struct and [MessagePeeker] trait
//!
//! [Message]: struct.Message.html
//! [Peek]: trait.MessagePeek.html
use async_trait::async_trait;
use lapin;

/// A zero-cost [lapin::message::Delivery] [newtype].
///
/// [lapin::message::Delivery]: https://docs.rs/lapin/latest/lapin/message/struct.Delivery.html
/// [newtype]: https://doc.rust-lang.org/1.0.0/style/features/types/newtype.html
pub struct Message(pub lapin::message::Delivery);

impl Message {
    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.0.data
    }
}

/// A trait to peek the [Message] data and returns the original data
/// or modified data.
///
/// [Message]: struct.Message.html
#[async_trait]
pub trait MessagePeeker {
    /// Async method to peek a message data.
    async fn peek(&mut self, msg: &Message) -> crate::Result<Vec<u8>>;
    fn boxed_clone(&self) -> Box<dyn MessagePeeker + Send>;
}

// https://users.rust-lang.org/t/solved-is-it-possible-to-clone-a-boxed-trait-object/1714/6
impl Clone for Box<dyn MessagePeeker + Send> {
    fn clone(&self) -> Box<dyn MessagePeeker + Send> {
        self.boxed_clone()
    }
}

/// A [MessagePeeker] implementation that echoes back the request message.
///
/// [MessagePeeker]: trait.MessagePeeker.html
#[derive(Clone)]
pub struct EchoPeeker;

#[async_trait]
impl MessagePeeker for EchoPeeker {
    /// Echoe back the request message.
    async fn peek(&mut self, msg: &Message) -> crate::Result<Vec<u8>> {
        Ok(msg.data().to_vec())
    }
    fn boxed_clone(&self) -> Box<dyn MessagePeeker + Send> {
        Box::new((*self).clone())
    }
}
