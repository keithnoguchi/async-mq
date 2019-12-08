// SPDX-License-Identifier: APACHE-2.0 AND MIT
//! ConsumerExt trait and some example concrete types.
use async_trait::async_trait;
use lapin;

#[async_trait]
pub trait ConsumerExt {
    async fn consume(&mut self, msg: Vec<u8>) -> lapin::Result<Vec<u8>>;
    fn box_clone(&self) -> Box<dyn ConsumerExt + Send>;
}

// https://users.rust-lang.org/t/solved-is-it-possible-to-clone-a-boxed-trait-object/1714/6
impl Clone for Box<dyn ConsumerExt + Send> {
    fn clone(&self) -> Box<dyn ConsumerExt + Send> {
        self.box_clone()
    }
}

#[derive(Clone)]
pub struct EchoConsumer;

#[async_trait]
impl ConsumerExt for EchoConsumer {
    async fn consume(&mut self, msg: Vec<u8>) -> lapin::Result<Vec<u8>> {
        Ok(msg)
    }
    fn box_clone(&self) -> Box<dyn ConsumerExt + Send> {
        Box::new((*self).clone())
    }
}
