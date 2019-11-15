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
                handle(sock)
                    .map(|_| ())
                    .map_err(|err| eprintln!("[{}]: handle error: {}", NAME, err)),
            );
            Ok(())
        })
}

fn handle(sock: TcpStream) -> impl Future<Item = (), Error = tokio::io::Error> {
    use tokio::io::AsyncRead;
    const NAME: &str = "echo::handle";
    let (reader, writer) = sock.split();
    println!("[{}]: reader {:?} and writer {:?}", NAME, reader, writer);
    tokio::io::copy(reader, writer).map(|_| ())
}
