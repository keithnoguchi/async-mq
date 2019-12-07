// SPDX-License-Identifier: GPL-2.0
//! Consumer crate and the example Noop consumer.
use futures::future::BoxFuture;
use futures_util::future::FutureExt;
use lapin;

pub trait Consumer<'future> {
    fn consume(msg: Vec<u8>) -> BoxFuture<'future, lapin::Result<()>>;
}

#[allow(dead_code)]
pub struct NoopConsumer;

impl<'future> Consumer<'future> for NoopConsumer {
    fn consume(_msg: Vec<u8>) -> BoxFuture<'future, lapin::Result<()>> {
        async { Ok(()) }.boxed()
    }
}
