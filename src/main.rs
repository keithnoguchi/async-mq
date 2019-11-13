// SPDX-License-Identifier: GPL-2.0
use tokio::prelude::*;
use tokio::{self, net};

fn main() {
    // https://tokio.rs/docs/getting-started/hello-world/
    let addr = "127.0.0.1:6142".parse().unwrap();
    let f = net::TcpStream::connect(&addr)
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
