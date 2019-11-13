// SPDX-License-Identifier: GPL-2.0
fn main() {
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    println!("Hello, world {}!", addr);
}
