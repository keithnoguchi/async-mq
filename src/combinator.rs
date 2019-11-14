// SPDX-License-Identifier: GPL-2.0
use futures;

pub struct HelloWorld;

impl futures::Future for HelloWorld {
    type Item = String;
    type Error = ();
    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        const NAME: &str = "combinator::HelloWorld";
        eprintln!("[{}] poll()", NAME);
        Ok(futures::Async::Ready(format!("[{}]: hello world", NAME)))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn map() {
        use futures::Future;
        let fut = super::HelloWorld;
        tokio::run(fut.map(|msg| println!("{}", msg)));
    }
}
