// SPDX-License-Identifier: GPL-2.0
use futures::{Async, Future, Poll};

pub struct Doubler<T> {
    inner: T,
}

pub fn double<T>(inner: T) -> Doubler<T> {
    Doubler { inner }
}

impl<T> Future for Doubler<T>
where
    T: Future<Item = usize>,
{
    type Item = usize;
    type Error = T::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.inner.poll()? {
            Async::Ready(v) => Ok(Async::Ready(v * 2)),
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn double() {
        use futures::{future::ok, Async, Future};
        assert_eq!(Async::Ready(1), ok::<usize, ()>(1).poll().unwrap());
    }
}
