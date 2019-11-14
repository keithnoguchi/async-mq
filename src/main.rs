// SPDX-License-Identifier: GPL-2.0
use std::thread;
use tokio::{self, prelude::*};

fn main() {
    let addr: std::net::SocketAddr = "127.0.0.1:6142".parse().unwrap();
    let addr1 = addr;
    let server = thread::spawn(move || {
        server(&addr);
    });
    let count = 1;
    let runtime = thread::spawn(move || {
        tokio::run(futures::future::lazy(move || {
            tokio::spawn(rustmq::basic::display(count));
            tokio::spawn(rustmq::basic::better_display(count));
            tokio::spawn(rustmq::peer::hello(&addr1));
            tokio::spawn(rustmq::peer::peer(&addr1));
            tokio::spawn(rustmq::combinator::hello());
            tokio::spawn(rustmq::echo::client(&addr1));
            tokio::spawn(rustmq::echo::client_and_then(&addr1));
            tokio::spawn(rustmq::echo::client_and_then_and_then(&addr1));
            Ok(())
        }))
    });
    runtime.join().unwrap();
    server.join().unwrap();
}

// https://tokio.rs/docs/getting-started/echo/
fn server(addr: &std::net::SocketAddr) {
    const NAME: &str = "server";
    let listener = tokio::net::TcpListener::bind(addr).unwrap();
    let fut = listener
        .incoming()
        .for_each(|sock| {
            println!("[{}]: connection from {:?}", NAME, sock);
            let (rx, tx) = sock.split();
            let amount = tokio::io::copy(rx, tx);
            let msg = amount.then(|ret| {
                match ret {
                    Ok((len, _, _)) => println!("[{}]: wrote {} bytes", NAME, len),
                    Err(err) => println!("[{}]: error: {}", NAME, err),
                }
                Ok(())
            });
            tokio::spawn(msg);
            Ok(())
        })
        .map_err(|err| {
            eprintln!("[{}]: accept error: {:?}", NAME, err);
        });
    println!("[{}]: server running on {}", NAME, addr);
    tokio::run(fut);
}
