// SPDX-License-Identifier: APACHE-2.0 AND MIT
//! Producer, ProducerBuilder, and ProducerExt trait.
use async_trait::async_trait;
use lapin;

#[async_trait]
pub trait ProducerExt {
    async fn recv(&mut self, msg: Vec<u8>) -> lapin::Result<()>;
    fn box_clone(&self) -> Box<dyn ProducerExt + Send>;
}

// https://users.rust-lang.org/t/solved-is-it-possible-to-clone-a-boxed-trait-object/1714/6
impl Clone for Box<dyn ProducerExt + Send> {
    fn clone(&self) -> Box<dyn ProducerExt + Send> {
        self.box_clone()
    }
}

#[derive(Clone)]
pub struct DebugPrintProducer;

#[async_trait]
impl ProducerExt for DebugPrintProducer {
    async fn recv(&mut self, msg: Vec<u8>) -> lapin::Result<()> {
        eprintln!("{:?}", msg);
        Ok(())
    }
    fn box_clone(&self) -> Box<dyn ProducerExt + Send> {
        Box::new((*self).clone())
    }
}
