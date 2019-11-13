// SPDX-License-Identifier: GPL-2.0
use std::thread;
use tokio::{self, prelude::*};

fn main() {
    let hello = thread::spawn(|| {
        hello();
    });
    let addr: std::net::SocketAddr = "127.0.0.1:6142".parse().unwrap();
    let client_addr = addr;
    let server = thread::spawn(move || {
        server(&addr);
    });
    let client = thread::spawn(move || {
        client(&client_addr);
    });
    hello.join().unwrap();
    client.join().unwrap();
    server.join().unwrap();
}

// https://tokio.rs/docs/futures/basic/
fn hello() {
    use rustmq::hello;
    let fut = hello::Display(hello::HelloWorld);
    tokio::run(fut);
    let fut = hello::BetterDisplay(hello::HelloWorld);
    tokio::run(fut);
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

// https://tokio.rs/docs/getting-started/hello-world/
fn client(addr: &std::net::SocketAddr) {
    const NAME: &str = "client";
    let fut = tokio::net::TcpStream::connect(addr)
        .and_then(|stream| {
            println!("created stream");
            tokio::io::write_all(stream, "hello world\n").then(|ret| {
                println!("[{}]: wrote to stream; success={:?}", NAME, ret.is_ok());
                Ok(())
            })
        })
        .map_err(|e| {
            println!("connection error: {:?}", e);
        });
    println!("[{}]: About to create the stream and write to it...", NAME);
    tokio::run(fut);
    println!("[{}]: Stream has been created and written to.", NAME);
}
