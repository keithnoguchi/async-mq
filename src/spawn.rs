// SPDX-License-Identifier: GPL-2.0
use futures::Future;
use std::net::SocketAddr;
use tokio;

// https://tokio.rs/docs/futures/spawning/
pub fn server(addr: &SocketAddr) -> impl Future<Item = (), Error = ()> {
    use futures::Stream;
    const NAME: &str = "spawn::server";
    tokio::net::tcp::TcpListener::bind(addr)
        .unwrap()
        .incoming()
        .for_each(|sock| {
            println!("[{}]: {:?}", NAME, sock);
            Ok(())
        })
        .map_err(|err| eprintln!("[{}]: {}", NAME, err))
}

#[cfg(test)]
mod tests {
    #[test]
    fn lazy() {
        struct Test {
            name: &'static str,
            count: usize,
        }
        let tests = [
            Test {
                name: "four lazy tasks",
                count: 4,
            },
            Test {
                name: "100 lazy tasks",
                count: 100,
            },
        ];
        for t in &tests {
            let name = t.name;
            let count = t.count;
            tokio::run(futures::future::lazy(move || {
                for i in 0..count {
                    tokio::spawn(futures::future::lazy(move || {
                        println!("[{}]: task #{}", name, i);
                        Ok(())
                    }));
                }
                Ok(())
            }));
        }
    }
    #[test]
    fn oneshot() {
        struct Test {
            name: &'static str,
            count: usize,
        }
        let tests = [
            Test {
                name: "single oneshot",
                count: 1,
            },
            Test {
                name: "100 oneshots",
                count: 100,
            },
            Test {
                name: "1,000 oneshots",
                count: 1_000,
            },
        ];
        for t in &tests {
            use futures::future::lazy;
            let name = t.name;
            let count = t.count;
            tokio::run(lazy(move || {
                use futures::{future::Future, sync::oneshot};
                for _ in 0..count {
                    let (tx, rx) = oneshot::channel();
                    tokio::spawn(lazy(move || {
                        tx.send(format!("{}", name))
                            .map_err(move |err| panic!("[{}] error: {}", name, err))
                    }));
                    tokio::spawn(lazy(move || {
                        rx.and_then(|msg| {
                            println!("[{}] got it!", msg);
                            Ok(())
                        })
                        .map_err(move |err| panic!("[{}] error: {}", name, err))
                    }));
                }
                Ok(())
            }))
        }
    }
    #[test]
    fn mpsc() {
        use futures::future::lazy;
        tokio::run(lazy(|| {
            use futures::sync::mpsc;
            use futures::{future::Future, sink::Sink, stream, Stream};
            let (tx, rx) = mpsc::channel(1_024);
            tokio::spawn({
                stream::iter_ok(0..10)
                    .fold(tx, |tx, i| {
                        tx.send(format!("message {} from spawned task", i))
                            .map_err(|err| eprintln!("error = {}", err))
                    })
                    .map(|_| ())
            });
            rx.for_each(|msg| {
                println!("Got {}", msg);
                Ok(())
            })
        }));
    }
}
