// SPDX-License-Identifier: GPL-2.0
use std::thread;
use tokio::{self, prelude::*};

fn main() {
    let addr1: std::net::SocketAddr = "127.0.0.1:6142".parse().unwrap();
    let addr2 = addr1;
    let server = thread::spawn(move || {
        server(&addr1);
    });
    let addr1 = addr2;
    let client = thread::spawn(move || {
        client(&addr2);
    });
    let basic = thread::spawn(|| {
        basic();
    });
    let addr2 = addr1;
    let peer = thread::spawn(move || {
        peer(&addr1);
    });
    let addr1 = addr2;
    let hello = thread::spawn(move || {
        hello(&addr2);
    });
    let mapper = thread::spawn(|| {
        mapper();
    });
    let addr2 = addr1;
    let and_then_thread = thread::spawn(move || {
        and_then_thread(&addr1);
    });
    let and_then_and_then_thread = thread::spawn(move || {
        and_then_and_then_thread(&addr2);
    });
    and_then_and_then_thread.join().unwrap();
    and_then_thread.join().unwrap();
    mapper.join().unwrap();
    hello.join().unwrap();
    peer.join().unwrap();
    basic.join().unwrap();
    client.join().unwrap();
    server.join().unwrap();
}

fn and_then_and_then_thread(addr: &std::net::SocketAddr) {
    const NAME: &str = "and_then_and_then_thread";
    let fut = tokio::net::tcp::TcpStream::connect(&addr)
        .and_then(|sock| tokio::io::write_all(sock, b"hello world"))
        .and_then(|(sock, _)| tokio::io::read_exact(sock, vec![0; 10]))
        .and_then(|(_, buf)| {
            println!("[{}]: got {:?}", NAME, buf);
            Ok(())
        })
        .map_err(|err| eprintln!("[{}]: error {}", NAME, err));
    tokio::run(fut);
}

fn and_then_thread(addr: &std::net::SocketAddr) {
    const NAME: &str = "and_then_thread";
    let fut = tokio::net::tcp::TcpStream::connect(&addr)
        .and_then(|sock| tokio::io::write_all(sock, b"hello world"))
        .map(|_| println!("[{}]: write complete", NAME))
        .map_err(|err| eprintln!("[{}]: write error: {}", NAME, err));
    tokio::run(fut);
}

// https://tokio.rs/docs/futures/combinators/
fn mapper() {
    const NAME: &str = "mapper";
    let fut = rustmq::combinator::HelloWorld;
    tokio::run(fut.map(|msg| println!("[{}]: {}", NAME, msg)));
}

// https://tokio.rs/docs/futures/getting_asynchronous/
fn hello(addr: &std::net::SocketAddr) {
    let conn = tokio::net::tcp::TcpStream::connect(&addr);
    let fut = rustmq::peer::HelloWorld::Connecting(conn);
    tokio::run(fut.map_err(|err| eprintln!("{0}", err)));
}

// https://tokio.rs/docs/futures/getting_asynchronous/
fn peer(addr: &std::net::SocketAddr) {
    let conn = tokio::net::tcp::TcpStream::connect(&addr);
    let fut = rustmq::peer::GetPeerAddr { conn };
    tokio::run(fut);
}

// https://tokio.rs/docs/futures/basic/
fn basic() {
    let count = 1;
    let fut = rustmq::basic::Display(rustmq::basic::HelloWorld::new(count));
    tokio::run(fut);
    let fut = rustmq::basic::BetterDisplay(rustmq::basic::HelloWorld::new(count));
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
            println!("[{}]: created stream", NAME);
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
