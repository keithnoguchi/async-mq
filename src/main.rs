// SPDX-License-Identifier: GPL-2.0
use std::{net, thread};
use tokio::{self, prelude::*};

fn main() {
    let addr: net::SocketAddr = "127.0.0.1:6142".parse().unwrap();
    let client_addr = addr;
    let server = thread::spawn(move || {
        server(&addr);
    });
    let client = thread::spawn(move || {
        client(&client_addr);
    });
    client.join().unwrap();
    server.join().unwrap();
}

// https://tokio.rs/docs/getting-started/echo/
fn server(addr: &net::SocketAddr) {
    let listener = tokio::net::TcpListener::bind(addr).unwrap();
    let f = listener
        .incoming()
        .for_each(|sock| {
            println!("connection from {:?}", sock);
            Ok(())
        })
        .map_err(|err| {
            eprintln!("accept error: {:?}", err);
        });
    println!("server running on {}", addr);
    tokio::run(f);
}

// https://tokio.rs/docs/getting-started/hello-world/
fn client(addr: &net::SocketAddr) {
    let f = tokio::net::TcpStream::connect(addr)
        .and_then(|stream| {
            println!("created stream");
            tokio::io::write_all(stream, "hello world\n").then(|r| {
                println!("wrote to stream; success={:?}", r.is_ok());
                Ok(())
            })
        })
        .map_err(|e| {
            println!("connection error: {:?}", e);
        });
    println!("About to create the stream and write to it...");
    tokio::run(f);
    println!("Stream has been created and written to.");
}
