// SPDX-License-Identifier: GPL-2.0
use futures::{Future, Stream};
use std::net::SocketAddr;
use tokio::net;

pub fn server(addr: &SocketAddr) -> impl Future<Item = (), Error = ()> {
    const NAME: &str = "echo::server";
    let l = net::TcpListener::bind(addr)
        .unwrap_or_else(|err| panic!(format!("[{}]: cannot bind: {}", NAME, err)));
    l.incoming()
        .map_err(|err| eprintln!("[{}]: accept failed: {}", NAME, err))
        .for_each(|sock| {
            println!("[{}]: from {:?}", NAME, sock);
            Ok(())
        })
}
