// SPDX-License-Identifier: GPL-2.0
use futures::{future, Future};
use std::net::SocketAddr;

pub fn server(_addr: &SocketAddr) -> impl Future<Item = (), Error = ()> {
    future::ok(())
}
