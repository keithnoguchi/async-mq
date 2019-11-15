// SPDX-License-Identifier: GPL-2.0
use futures::{Future, Stream};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

pub fn server(addr: &SocketAddr) -> impl Future<Item = (), Error = ()> {
    const NAME: &str = "echo::server";
    let l = TcpListener::bind(addr)
        .unwrap_or_else(|err| panic!(format!("[{}]: cannot bind: {}", NAME, err)));
    l.incoming()
        .map_err(|err| eprintln!("[{}]: accept failed: {}", NAME, err))
        .for_each(|sock| {
            let peer = sock.peer_addr().unwrap();
            tokio::spawn(
                handle(sock)
                    .map_err(|err| eprintln!("[{}]: handle error: {}", NAME, err))
                    .map(move |num| {
                        println!("[{}]: {:?} bytes written total to {}", NAME, num, peer)
                    }),
            );
            Ok(())
        })
}

fn handle(sock: TcpStream) -> impl Future<Item = u64, Error = tokio::io::Error> {
    use tokio::io::AsyncRead;
    let (reader, writer) = sock.split();
    tokio::io::copy(reader, writer).map(|(num, _, _)| num)
}
