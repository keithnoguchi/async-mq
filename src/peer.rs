// SPDX-License-Identifier: GPL-2.0
use bytes;
use futures::{self, Future};
use tokio;

// https://tokio.rs/docs/futures/getting_asynchronous/
pub enum HelloWorld {
    Connecting(tokio::net::tcp::ConnectFuture),
    Connected(tokio::net::tcp::TcpStream, std::io::Cursor<bytes::Bytes>),
}

impl futures::Future for HelloWorld {
    type Item = ();
    type Error = std::io::Error;
    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        use bytes::Buf;
        use tokio::io::AsyncWrite;
        const NAME: &str = "peer::HelloWorld";
        println!("[{}]: poll()", NAME);
        loop {
            match self {
                Self::Connecting(ref mut f) => {
                    let sock = futures::try_ready!(f.poll());
                    let data = std::io::Cursor::new(bytes::Bytes::from_static(b"hello world"));
                    println!("[{}]: Connecting", NAME);
                    *self = Self::Connected(sock, data);
                }
                Self::Connected(ref mut sock, ref mut data) => {
                    println!("[{}]: Connected", NAME);
                    while data.has_remaining() {
                        futures::try_ready!(sock.write_buf(data));
                    }
                    return Ok(futures::Async::Ready(()));
                }
            }
        }
    }
}

pub struct GetPeerAddr {
    pub conn: tokio::net::tcp::ConnectFuture,
}

impl futures::Future for GetPeerAddr {
    type Item = ();
    type Error = ();
    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        const NAME: &str = "peer::GetPeerAddr";
        match self.conn.poll() {
            Ok(futures::Async::Ready(sock)) => {
                println!("[{}]: peer address = {}", NAME, sock.peer_addr().unwrap());
                Ok(futures::Async::Ready(()))
            }
            Ok(futures::Async::NotReady) => {
                eprintln!("[{}]: NotReady", NAME);
                Ok(futures::Async::NotReady)
            }
            Err(err) => {
                eprintln!("[{}]: failed to connect: {}", NAME, err);
                Ok(futures::Async::Ready(()))
            }
        }
    }
}

pub fn hello(addr: &std::net::SocketAddr) -> impl Future<Item = (), Error = ()> {
    let conn = tokio::net::tcp::TcpStream::connect(&addr);
    HelloWorld::Connecting(conn).map_err(|err| eprintln!("{0}", err))
}

pub fn peer(addr: &std::net::SocketAddr) -> impl Future<Item = (), Error = ()> {
    let conn = tokio::net::tcp::TcpStream::connect(&addr);
    GetPeerAddr { conn }
}
