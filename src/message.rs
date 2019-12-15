// SPDX-License-Identifier: APACHE-2.0 AND MIT
//! [Message] struct, [MessagePeeker] and [MessageProcessor] trait
//!
//! [Message]: struct.Message.html
//! [MessagePeeker]: trait.MessagePeeker.html
//! [MessageProcessor]: trait.MessageProcessor.html
use async_trait::async_trait;
use lapin;

/// A zero-cost [lapin::message::Delivery] [newtype].
///
/// [lapin::message::Delivery]: https://docs.rs/lapin/latest/lapin/message/struct.Delivery.html
/// [newtype]: https://doc.rust-lang.org/1.0.0/style/features/types/newtype.html
pub struct Message(pub lapin::message::Delivery);

/// Error actions used both by [MessagePeeker] and [MessageProcessor]
/// trait implementations.
///
/// [MessagePeeker]: trait.MessagePeeker.html
/// [MessageProcessor]: trait.MessageProcessor.html
pub enum MessageError {
    /// Silently drops the message.
    Drop,
    /// Reject a message.
    Reject,
    /// Nack a message.
    Nack,
}

impl Message {
    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.0.data
    }
}

/// A trait to peek the [Message] and returns success or error.
///
/// [Message]: struct.Message.html
#[async_trait]
pub trait MessagePeeker {
    /// Async method to peek a message.
    async fn peek(&mut self, msg: &Message) -> Result<(), MessageError>;
    fn boxed_clone(&self) -> Box<dyn MessagePeeker + Send>;
}

// https://users.rust-lang.org/t/solved-is-it-possible-to-clone-a-boxed-trait-object/1714/6
impl Clone for Box<dyn MessagePeeker + Send> {
    fn clone(&self) -> Box<dyn MessagePeeker + Send> {
        self.boxed_clone()
    }
}

/// A trait to process the [Message] and returns the response data
/// or modified data.
///
/// [Message]: struct.Message.html
#[async_trait]
pub trait MessageProcessor {
    /// Async method to process a message.
    async fn process(&mut self, msg: &Message) -> Result<Vec<u8>, MessageError>;
    fn boxed_clone(&self) -> Box<dyn MessageProcessor + Send>;
}

// https://users.rust-lang.org/t/solved-is-it-possible-to-clone-a-boxed-trait-object/1714/6
impl Clone for Box<dyn MessageProcessor + Send> {
    fn clone(&self) -> Box<dyn MessageProcessor + Send> {
        self.boxed_clone()
    }
}

/// A [MessagePeeker] implementation which does nothing.
///
/// [MessagePeeker]: trait.MessagePeeker.html
#[derive(Clone)]
pub struct NoopPeeker;

#[async_trait]
impl MessagePeeker for NoopPeeker {
    /// Echoe back the request message.
    async fn peek(&mut self, _msg: &Message) -> Result<(), MessageError> {
        Ok(())
    }
    fn boxed_clone(&self) -> Box<dyn MessagePeeker + Send> {
        Box::new((*self).clone())
    }
}

/// A [MessagePeeker] implementation which reject a message.
///
/// [MessagePeeker]: trait.MessagePeeker.html
#[derive(Clone)]
struct RejectPeeker;

#[async_trait]
impl MessagePeeker for RejectPeeker {
    /// Just returns the error saying to drop a message.
    /// to the console.  This is good for the benchmarking.
    async fn peek(&mut self, _msg: &Message) -> Result<(), MessageError> {
        Err(MessageError::Reject)
    }
    fn boxed_clone(&self) -> Box<dyn MessagePeeker + Send> {
        Box::new((*self).clone())
    }
}

/// A [MessageProcessor] implementation which echoes back the original message.
///
/// [MessageProcessor]: trait.MessageProcessor.html
#[derive(Clone)]
pub struct EchoProcessor;

#[async_trait]
impl MessageProcessor for EchoProcessor {
    /// Echoe back the request message.
    async fn process(&mut self, msg: &Message) -> Result<Vec<u8>, MessageError> {
        Ok(msg.data().to_vec())
    }
    fn boxed_clone(&self) -> Box<dyn MessageProcessor + Send> {
        Box::new((*self).clone())
    }
}
