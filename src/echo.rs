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
            handle(sock)
                .map(|ret| ret)
                .map_err(|err| eprintln!("[{}]: handle error: {}", NAME, err))
        })
}

fn handle(sock: TcpStream) -> Result<(), tokio::io::Error> {
    use tokio::io::AsyncRead;
    const NAME: &str = "echo::handle";
    let (reader, writer) = sock.split();
    println!("[{}]: reader {:?} and writer {:?}", NAME, reader, writer);
    tokio::spawn(tokio::io::copy(reader, writer).map(|_| ()).map_err(|_| ()));
    Ok(())
}
