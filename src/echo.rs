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
            tokio::spawn(
                greeting(sock)
                    .map_err(|err| eprintln!("[{}]: greeting error: {}", NAME, err))
                    .map(|sock| {
                        tokio::spawn(
                            handle(sock)
                                .map_err(|err| eprintln!("[{}]: handle error: {}", NAME, err))
                                .map(|sock| {
                                    let peer = sock.peer_addr().unwrap();
                                    println!("[{}]: taken care of {}", NAME, peer);
                                }),
                        );
                    }),
            );
            Ok(())
        })
}

fn greeting(sock: TcpStream) -> impl Future<Item = TcpStream, Error = tokio::io::Error> {
    tokio::io::write_all(sock, "Howdy!\n").map(|(sock, _)| sock)
}

fn handle(sock: TcpStream) -> impl Future<Item = TcpStream, Error = tokio::io::Error> {
    use tokio::io::AsyncRead;
    let (reader, writer) = sock.split();
    tokio::io::copy(reader, writer).map(|(_, r, w)| r.unsplit(w))
}
