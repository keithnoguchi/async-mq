// SPDX-License-Identifier: APACHE-2.0 AND MIT
//! Producer trait and some example concrete types.
use async_trait::async_trait;
use lapin;

#[async_trait]
pub trait Producer {
    async fn recv(&mut self, msg: Vec<u8>) -> lapin::Result<()>;
    fn box_clone(&self) -> Box<dyn Producer + Send>;
}

// https://users.rust-lang.org/t/solved-is-it-possible-to-clone-a-boxed-trait-object/1714/6
impl Clone for Box<dyn Producer + Send> {
    fn clone(&self) -> Box<dyn Producer + Send> {
        self.box_clone()
    }
}

#[derive(Clone)]
pub struct DebugPrintProducer;

#[async_trait]
impl Producer for DebugPrintProducer {
    async fn recv(&mut self, msg: Vec<u8>) -> lapin::Result<()> {
        eprintln!("{:?}", msg);
        Ok(())
    }
    fn box_clone(&self) -> Box<dyn Producer + Send> {
        Box::new((*self).clone())
    }
}
