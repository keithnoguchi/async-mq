// SPDX-License-Identifier: GPL-2.0
use futures::Future;
use std::net::SocketAddr;
use tokio::{self, net::tcp::TcpStream};

// https://tokio.rs/docs/getting-started/hello-world/
pub fn client(addr: &SocketAddr) -> impl Future<Item = (), Error = ()> {
    const NAME: &str = "echo::client";
    TcpStream::connect(addr)
        .and_then(|stream| {
            println!("[{}]: created stream", NAME);
            tokio::io::write_all(stream, "hello world\n").then(|ret| {
                println!("[{}]: wrote to stream; success={:?}", NAME, ret.is_ok());
                Ok(())
            })
        })
        .map_err(|e| {
            println!("connection error: {:?}", e);
        })
}

// https://tokio.rs/docs/futures/combinators/
pub fn client_and_then(addr: &SocketAddr) -> impl Future<Item = (), Error = ()> {
    const NAME: &str = "echo::client_and_then";
    TcpStream::connect(addr)
        .and_then(|sock| tokio::io::write_all(sock, b"hello world"))
        .map(|_| println!("[{}]: write complete", NAME))
        .map_err(|err| eprintln!("[{}]: write error: {}", NAME, err))
}

// https://tokio.rs/docs/futures/combinators/
pub fn client_and_then_and_then(addr: &SocketAddr) -> impl Future<Item = (), Error = ()> {
    const NAME: &str = "echo::client_and_then_and_then";
    TcpStream::connect(addr)
        .and_then(|sock| tokio::io::write_all(sock, b"hello world"))
        .and_then(|(sock, _)| tokio::io::read_exact(sock, vec![0; 10]))
        .and_then(|(_, buf)| {
            println!("[{}]: got {:?}", NAME, buf);
            Ok(())
        })
        .map_err(|err| eprintln!("[{}]: error {}", NAME, err))
}
