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

pub fn background(addr: &SocketAddr) -> impl Future<Item = (), Error = ()> {
    let addr = *addr;
    futures::future::lazy(move || {
        use futures::Stream;
        const NAME: &str = "spawn::background";
        let (_tx, rx) = tokio::sync::mpsc::channel(1_024);
        tokio::spawn(work(rx));
        tokio::net::tcp::TcpListener::bind(&addr)
            .unwrap()
            .incoming()
            .for_each(|peer| {
                println!("[{}]: got {} request", NAME, peer.peer_addr().unwrap());
                Ok(())
            })
            .map_err(|err| eprintln!("[{}]: {:?}", NAME, err))
    })
}

fn work(_rx: tokio::sync::mpsc::Receiver<usize>) -> impl Future<Item = (), Error = ()> {
    println!("doing nothing for now...");
    futures::future::ok::<(), ()>(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn work() {
        use futures::Future;
        struct Test {
            name: &'static str,
            bufsiz: usize,
        }
        let tests = [
            Test {
                name: "one byte buffer",
                bufsiz: 1,
            },
            Test {
                name: "two bytes buffer",
                bufsiz: 2,
            },
            Test {
                name: "512 bytes buffer",
                bufsiz: 512,
            },
            Test {
                name: "1,024 bytes buffer",
                bufsiz: 1_024,
            },
            Test {
                name: "4K bytes buffer",
                bufsiz: 4_094,
            },
        ];
        for t in &tests {
            let (_, rx) = tokio::sync::mpsc::channel(t.bufsiz);
            debug_assert_eq!(Ok(()), super::work(rx).wait(), "{}", t.name);
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
