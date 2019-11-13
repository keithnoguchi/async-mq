// SPDX-License-Identifier: GPL-2.0
// https://tokio.rs/docs/futures/getting_asynchronous/
use futures;
use tokio;

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
