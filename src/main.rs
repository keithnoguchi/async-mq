// SPDX-License-Identifier: GPL-2.0
use futures;
use tokio;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:6142".parse::<std::net::SocketAddr>()?;
    let addr2 = "127.0.0.1:6143".parse::<std::net::SocketAddr>()?;
    tokio::run(futures::future::lazy(move || {
        let count = 1;
        tokio::spawn(rustmq::echo::server(&addr));
        tokio::spawn(rustmq::echo::client(&addr));
        tokio::spawn(rustmq::echo::client_and_then(&addr));
        tokio::spawn(rustmq::echo::client_and_then_and_then(&addr));
        tokio::spawn(rustmq::basic::display(count));
        tokio::spawn(rustmq::basic::better_display(count));
        tokio::spawn(rustmq::peer::hello(&addr));
        tokio::spawn(rustmq::peer::peer(&addr));
        tokio::spawn(rustmq::combinator::hello());
        tokio::spawn(rustmq::spawn::server(&addr2));
        Ok(())
    }));
    Ok(())
}
