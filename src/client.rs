// SPDX-License-Identifier: GPL-2.0
use futures::Future;

// https://tokio.rs/docs/getting-started/hello-world/
pub fn and_then(addr: &std::net::SocketAddr) -> impl Future<Item = (), Error = ()> {
    const NAME: &str = "client::and_then";
    tokio::net::tcp::TcpStream::connect(&addr)
        .and_then(|sock| tokio::io::write_all(sock, b"hello world"))
        .map(|_| println!("[{}]: write complete", NAME))
        .map_err(|err| eprintln!("[{}]: write error: {}", NAME, err))
}

pub fn and_then_and_then(addr: &std::net::SocketAddr) -> impl Future<Item = (), Error = ()> {
    const NAME: &str = "client::and_then_and_then";
    tokio::net::tcp::TcpStream::connect(&addr)
        .and_then(|sock| tokio::io::write_all(sock, b"hello world"))
        .and_then(|(sock, _)| tokio::io::read_exact(sock, vec![0; 10]))
        .and_then(|(_, buf)| {
            println!("[{}]: got {:?}", NAME, buf);
            Ok(())
        })
        .map_err(|err| eprintln!("[{}]: error {}", NAME, err))
}
