// SPDX-License-Identifier: GPL-2.0
use futures;
use tokio;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:6141".parse::<std::net::SocketAddr>()?;
    let addr2 = "127.0.0.1:6142".parse::<std::net::SocketAddr>()?;
    let addr3 = "127.0.0.1:6143".parse::<std::net::SocketAddr>()?;
    tokio::run(futures::future::lazy(move || {
        let client_count = 512;
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
        for _ in 0..client_count {
            tokio::spawn(rustmq::echo::client(&addr2));
        }
        tokio::spawn(rustmq::spawn::background(&addr3));
        // background server takes a while to setup,
        // and echo::client doesn't have a retry capability
        // yet.
        std::thread::sleep(std::time::Duration::from_millis(10));
        for _ in 0..client_count {
            tokio::spawn(rustmq::echo::client(&addr3));
        }
        Ok(())
    }));
    Ok(())
}
