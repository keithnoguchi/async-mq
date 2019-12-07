// SPDX-License-Identifier: GPL-2.0
//! Consumer crate and the example Noop consumer.
use futures::future::BoxFuture;
use futures_util::future::FutureExt;
use lapin;

pub trait Consumer<'future> {
    type Output;
    fn consume(msg: Vec<u8>) -> BoxFuture<'future, Self::Output>;
}

#[allow(dead_code)]
pub struct NoopConsumer;

impl<'future> Consumer<'future> for NoopConsumer {
    type Output = lapin::Result<()>;
    fn consume(_msg: Vec<u8>) -> BoxFuture<'future, Self::Output> {
        async { Ok(()) }.boxed()
    }
}

#[allow(dead_code)]
pub struct EchoConsumer;

impl<'future> Consumer<'future> for EchoConsumer {
    type Output = lapin::Result<()>;
    fn consume(_msg: Vec<u8>) -> BoxFuture<'future, Self::Output> {
        async { Ok(()) }.boxed()
    }
}
