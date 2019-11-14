// SPDX-License-Identifier: GPL-2.0
use futures;

// https://tokio.rs/docs/futures/streams/
pub struct Fibonacci {
    curr: u64,
    next: u64,
}

impl Fibonacci {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Fibonacci {
    fn default() -> Self {
        Self { curr: 1, next: 1 }
    }
}

impl futures::Stream for Fibonacci {
    type Item = u64;
    type Error = ();
    fn poll(&mut self) -> futures::Poll<Option<u64>, ()> {
        let curr = self.curr;
        let next = curr + self.next;
        self.curr = self.next;
        self.next = next;
        Ok(futures::Async::Ready(Some(curr)))
    }
}

pub struct Display<T> {
    stream: T,
    curr: usize,
    max: usize,
}

impl<T> Display<T> {
    pub fn new(stream: T, max: usize) -> Self {
        Self {
            stream,
            curr: 0,
            max,
        }
    }
}

impl<T> futures::Future for Display<T>
where
    T: futures::Stream,
    T::Item: std::fmt::Display,
{
    type Item = ();
    type Error = T::Error;
    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        while self.curr < self.max {
            let value = match futures::try_ready!(self.stream.poll()) {
                Some(value) => value,
                None => break,
            };
            println!("value #{} = {}", self.curr, value);
            self.curr += 1;
        }
        Ok(futures::Async::Ready(()))
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn display10() {
        struct Test {
            _name: &'static str,
            max: usize,
        }
        let tests = [
            Test {
                _name: "print out 1 fibonacci number",
                max: 1,
            },
            Test {
                _name: "print out 5 fibonacci numbers",
                max: 5,
            },
            Test {
                _name: "print out 10 fibonacci numbers",
                max: 10,
            },
        ];
        for t in &tests {
            let fib = super::Fibonacci::new();
            let stream = super::Display::new(fib, t.max);
            tokio::run(stream);
        }
    }
}
