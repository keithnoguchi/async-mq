// SPDX-License-Identifier: GPL-2.0
use futures::{self, future, sync, Future};
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
            let peer = sock.peer_addr().unwrap();
            tokio::spawn({
                println!("[{}]: handling {}", NAME, sock.peer_addr().unwrap());
                tokio::io::write_all(sock, "hello world")
                    // Drop the socket
                    .map(|_| ())
                    .map_err(|err| eprintln!("[{}]: {:?}", NAME, err))
            });
            println!("[{}]: spawned {} handler", NAME, peer);
            Ok(())
        })
        .map_err(|err| eprintln!("[{}]: {}", NAME, err))
}

// Background processing example explained in
// https://tokio.rs/docs/futures/spawning/
pub fn background(addr: &SocketAddr) -> impl Future<Item = (), Error = ()> {
    const NAME: &str = "spawn::background";
    let addr = *addr;
    future::lazy(move || {
        use futures::{Sink, Stream};
        let (tx, rx) = sync::mpsc::channel(1_024);
        tokio::spawn(sum(rx));
        println!("[{}] listen on {:?}", NAME, addr);
        tokio::net::tcp::TcpListener::bind(&addr)
            .unwrap()
            .incoming()
            .for_each(move |sock| {
                println!("[{}]: from {}", NAME, sock.peer_addr().unwrap());
                tokio::spawn({
                    let tx = tx.clone();
                    tokio::io::read_to_end(sock, vec![])
                        .and_then(move |(_, buf)| {
                            tx.send(buf.len())
                                .map_err(|_| tokio::io::ErrorKind::Other.into())
                        })
                        .map(|_| ())
                        .map_err(|err| eprintln!("[{}]: {:?}", NAME, err))
                });
                Ok(())
            })
            .map_err(|err| eprintln!("[{}]: {:?}", NAME, err))
    })
}

fn sum(rx: sync::mpsc::Receiver<usize>) -> impl Future<Item = (), Error = ()> {
    use futures::Stream;
    const NAME: &str = "spawn::sum";
    #[derive(Eq, PartialEq)]
    enum Item {
        Value(usize),
        Tick,
        Done,
    }
    // summary interval tick(5sec).
    let tick_dur = std::time::Duration::from_secs(5);
    let interval = tokio::timer::Interval::new_interval(tick_dur)
        .map(|_| Item::Tick)
        .map_err(|err| eprintln!("[{}]: {}", NAME, err));
    // Turn the stream into a sequence of:
    // Item(Value), Item(Value), Tick, Item(Value)... Done.
    let items = rx
        .map(Item::Value)
        .chain(futures::stream::once(Ok(Item::Done)))
        .select(interval)
        .take_while(|item| future::ok(*item != Item::Done));
    // our logic future.
    items
        .fold(0, |num, item| match item {
            Item::Value(v) => future::ok(num + v),
            Item::Tick => {
                println!("[{}]: bytes read = {}", NAME, num);
                future::ok(0)
            }
            _ => unreachable!(),
        })
        .map(|_| ())
}

// Coordinating access to a resource example explained in
// https://tokio.rs/docs/futures/spawning/
pub fn coordinate(requesters: usize) -> impl Future<Item = (), Error = ()> {
    future::lazy(move || {
        let (tx, rx) = sync::mpsc::channel(1_024);
        for _ in 0..requesters {
            tokio::spawn(ping(tx.clone()));
        }
        tokio::spawn(pong(rx));
        Ok(())
    })
}

fn ping(tx: sync::mpsc::Sender<u32>) -> impl Future<Item = (), Error = ()> {
    const NAME: &str = "spawn::ping";
    println!("[{}] {:?}", NAME, tx);
    future::ok(())
}

fn pong(rx: sync::mpsc::Receiver<u32>) -> impl Future<Item = (), Error = ()> {
    const NAME: &str = "spawn::pong";
    println!("[{}] {:?}", NAME, rx);
    future::ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn sum() {
        #[derive(Clone)]
        struct Test {
            name: &'static str,
            bufsiz: usize,
            producers: usize,
            data: usize,
        }
        let tests = [
            Test {
                name: "one producer on one byte channel buffer",
                bufsiz: 1,
                producers: 1,
                data: 256,
            },
            Test {
                name: "one producer on two bytes channel buffer",
                bufsiz: 2,
                producers: 1,
                data: 256,
            },
            Test {
                name: "one producer on 512 bytes channel buffer",
                bufsiz: 512,
                producers: 1,
                data: 256,
            },
            Test {
                name: "one producer on 1,024 bytes channel buffer",
                bufsiz: 1_024,
                producers: 1,
                data: 256,
            },
            Test {
                name: "one producer on 4K bytes channel buffer",
                bufsiz: 4_094,
                producers: 1,
                data: 256,
            },
            Test {
                name: "two producers on one byte channel buffer",
                bufsiz: 1,
                producers: 2,
                data: 256,
            },
            Test {
                name: "two producers on two bytes channel buffer",
                bufsiz: 2,
                producers: 2,
                data: 256,
            },
            Test {
                name: "two producers on 512 bytes channel buffer",
                bufsiz: 512,
                producers: 2,
                data: 256,
            },
            Test {
                name: "two producers on 1,024 bytes channel buffer",
                bufsiz: 1_024,
                producers: 2,
                data: 256,
            },
            Test {
                name: "two producers on 4K bytes channel buffer",
                bufsiz: 4_094,
                producers: 2,
                data: 256,
            },
            Test {
                name: "four producers on one byte channel buffer",
                bufsiz: 1,
                producers: 4,
                data: 256,
            },
            Test {
                name: "four producers on two bytes channel buffer",
                bufsiz: 2,
                producers: 4,
                data: 256,
            },
            Test {
                name: "four producers on 512 bytes channel buffer",
                bufsiz: 512,
                producers: 4,
                data: 256,
            },
            Test {
                name: "four producers on 1,024 bytes channel buffer",
                bufsiz: 1_024,
                producers: 4,
                data: 256,
            },
            Test {
                name: "four producers on 4K bytes channel buffer",
                bufsiz: 4_094,
                producers: 4,
                data: 256,
            },
            Test {
                name: "16 producers on one byte channel buffer",
                bufsiz: 1,
                producers: 16,
                data: 256,
            },
            Test {
                name: "16 producers on two bytes channel buffer",
                bufsiz: 2,
                producers: 16,
                data: 256,
            },
            Test {
                name: "16 producers on 512 bytes channel buffer",
                bufsiz: 512,
                producers: 16,
                data: 256,
            },
            Test {
                name: "16 producers on 1,024 bytes channel buffer",
                bufsiz: 1_024,
                producers: 16,
                data: 256,
            },
            Test {
                name: "16 producers on 4K bytes channel buffer",
                bufsiz: 4_094,
                producers: 16,
                data: 256,
            },
            Test {
                name: "64 producers on one byte channel buffer",
                bufsiz: 1,
                producers: 64,
                data: 256,
            },
            Test {
                name: "64 producers on two bytes channel buffer",
                bufsiz: 2,
                producers: 64,
                data: 256,
            },
            Test {
                name: "64 producers on 512 bytes channel buffer",
                bufsiz: 512,
                producers: 64,
                data: 256,
            },
            Test {
                name: "64 producers on 1,024 bytes channel buffer",
                bufsiz: 1_024,
                producers: 64,
                data: 256,
            },
            Test {
                name: "64 producers on 4K bytes channel buffer",
                bufsiz: 4_094,
                producers: 64,
                data: 256,
            },
            Test {
                name: "256 producers on one byte channel buffer",
                bufsiz: 1,
                producers: 256,
                data: 256,
            },
            Test {
                name: "256 producers on two bytes channel buffer",
                bufsiz: 2,
                producers: 256,
                data: 256,
            },
            Test {
                name: "256 producers on 512 bytes channel buffer",
                bufsiz: 512,
                producers: 256,
                data: 256,
            },
            Test {
                name: "256 producers on 1,024 bytes channel buffer",
                bufsiz: 1_024,
                producers: 256,
                data: 256,
            },
            Test {
                name: "256 producers on 4K bytes channel buffer",
                bufsiz: 4_094,
                producers: 256,
                data: 256,
            },
        ];
        for t in &tests {
            let t = t.clone();
            tokio::run(futures::future::lazy(move || {
                use futures::{Future, Sink};
                let (tx, rx) = futures::sync::mpsc::channel(t.bufsiz);
                for _ in 0..t.producers {
                    tokio::spawn({
                        tx.clone()
                            .send(t.data)
                            .map(|_| ())
                            .map_err(|err| eprintln!("{}", err))
                    });
                }
                tokio::spawn(super::sum(rx));
                println!("{}: done", t.name);
                Ok(())
            }));
        }
    }
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
                use futures::Future;
                for _ in 0..count {
                    let (tx, rx) = futures::sync::oneshot::channel();
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
            use futures::{future::Future, sink::Sink, stream, Stream};
            let (tx, rx) = futures::sync::mpsc::channel(1_024);
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
