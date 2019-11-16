// SPDX-License-Identifier: GPL-2.0
use futures::{Async, Future, Poll};

#[derive(Debug)]
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
    fn double_poll_ok() {
        use futures::{Async, Future};
        struct Test {
            name: &'static str,
            data: usize,
            want: Async<usize>,
        };
        let tests = [
            Test {
                name: "1usize",
                data: 1,
                want: Async::Ready(2),
            },
            Test {
                name: "2usize",
                data: 2,
                want: Async::Ready(4),
            },
            Test {
                name: "16usize",
                data: 16,
                want: Async::Ready(32),
            },
            Test {
                name: "10_001usize",
                data: 10_001,
                want: Async::Ready(20_002),
            },
        ];
        for t in &tests {
            let got = super::double(futures::future::ok::<usize, ()>(t.data))
                .poll()
                .unwrap();
            debug_assert_eq!(t.want, got, "{}", t.name);
        }
    }
}
