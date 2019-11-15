// SPDX-License-Identifier: GPL-2.0
use futures;
use tokio;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut addrs = Vec::<std::net::SocketAddr>::new();
    for i in 0..4 {
        let addr = format!("127.0.0.1:614{}", i).parse()?;
        addrs.push(addr);
    }
    tokio::run(futures::future::lazy(move || {
        let client_count = 512;
        let count = 1;
        tokio::spawn(rustmq::hello::server(&addrs[0]));
        tokio::spawn(rustmq::hello::client(&addrs[0]));
        tokio::spawn(rustmq::hello::client_and_then(&addrs[0]));
        tokio::spawn(rustmq::hello::client_and_then_and_then(&addrs[0]));
        tokio::spawn(rustmq::basic::display(count));
        tokio::spawn(rustmq::basic::better_display(count));
        tokio::spawn(rustmq::peer::hello(&addrs[0]));
        tokio::spawn(rustmq::peer::peer(&addrs[0]));
        tokio::spawn(rustmq::combinator::hello());
        tokio::spawn(rustmq::spawn::server(&addrs[1]));
        for _ in 0..client_count {
            tokio::spawn(rustmq::hello::client(&addrs[1]));
        }
        tokio::spawn(rustmq::spawn::background(&addrs[2]));
        // background server takes a while to setup,
        // and hello::client doesn't have a retry capability
        // yet.
        std::thread::sleep(std::time::Duration::from_millis(10));
        for _ in 0..client_count {
            tokio::spawn(rustmq::hello::client(&addrs[2]));
        }
        tokio::spawn(rustmq::spawn::coordinate(client_count));
        // https://tokio.rs/docs/io/overview/
        tokio::spawn(rustmq::echo::server(&addrs[3]));
        for _ in 0..client_count {
            tokio::spawn(rustmq::hello::client(&addrs[3]));
        }
        Ok(())
    }));
    Ok(())
}
