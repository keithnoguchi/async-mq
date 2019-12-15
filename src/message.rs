// SPDX-License-Identifier: APACHE-2.0 AND MIT
//! `Message` struct, `MessagePeek` and `MessageProcess` trait
use async_trait::async_trait;
use lapin;

/// A zero-cost [lapin::message::Delivery] [newtype].
///
/// [lapin::message::Delivery]: https://docs.rs/lapin/latest/lapin/message/struct.Delivery.html
/// [newtype]: https://doc.rust-lang.org/1.0.0/style/features/types/newtype.html
pub struct Message(pub lapin::message::Delivery);

/// Error actions used both by [MessagePeek] and [MessageProcess]
/// trait implementations.
///
/// [MessagePeek]: trait.MessagePeek.html
/// [MessageProcess]: trait.MessageProcess.html
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
pub trait MessagePeek {
    /// Async method to peek a message.
    async fn peek(&mut self, msg: &Message) -> Result<(), MessageError>;
    fn boxed_clone(&self) -> Box<dyn MessagePeek + Send>;
}

// https://users.rust-lang.org/t/solved-is-it-possible-to-clone-a-boxed-trait-object/1714/6
impl Clone for Box<dyn MessagePeek + Send> {
    fn clone(&self) -> Box<dyn MessagePeek + Send> {
        self.boxed_clone()
    }
}

/// A trait to process the [Message] and returns the response data
/// or modified data.
///
/// [Message]: struct.Message.html
#[async_trait]
pub trait MessageProcess {
    /// Async method to process a message.
    async fn process(&mut self, msg: &Message) -> Result<Vec<u8>, MessageError>;
    fn boxed_clone(&self) -> Box<dyn MessageProcess + Send>;
}

// https://users.rust-lang.org/t/solved-is-it-possible-to-clone-a-boxed-trait-object/1714/6
impl Clone for Box<dyn MessageProcess + Send> {
    fn clone(&self) -> Box<dyn MessageProcess + Send> {
        self.boxed_clone()
    }
}

/// A [MessagePeek] implementation which does nothing.
///
/// [MessagePeek]: trait.MessagePeek.html
#[derive(Clone)]
pub struct NoopPeeker;

#[async_trait]
impl MessagePeek for NoopPeeker {
    /// Echoe back the request message.
    async fn peek(&mut self, _msg: &Message) -> Result<(), MessageError> {
        Ok(())
    }
    fn boxed_clone(&self) -> Box<dyn MessagePeek + Send> {
        Box::new((*self).clone())
    }
}

/// A [MessagePeek] implementation which reject a message.
///
/// [MessagePeek]: trait.MessagePeek.html
#[derive(Clone)]
struct RejectPeeker;

#[async_trait]
impl MessagePeek for RejectPeeker {
    /// Just returns the error saying to drop a message.
    /// to the console.  This is good for the benchmarking.
    async fn peek(&mut self, _msg: &Message) -> Result<(), MessageError> {
        Err(MessageError::Reject)
    }
    fn boxed_clone(&self) -> Box<dyn MessagePeek + Send> {
        Box::new((*self).clone())
    }
}

/// A [MessageProcess] implementation which echoes back the original message.
///
/// [MessageProcess]: trait.MessageProcess.html
#[derive(Clone)]
pub struct EchoProcessor;

#[async_trait]
impl MessageProcess for EchoProcessor {
    /// Echoe back the request message.
    async fn process(&mut self, msg: &Message) -> Result<Vec<u8>, MessageError> {
        Ok(msg.data().to_vec())
    }
    fn boxed_clone(&self) -> Box<dyn MessageProcess + Send> {
        Box::new((*self).clone())
    }
}
