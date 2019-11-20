// SPDX-License-Identifier: GPL-2.0
use lapin::options::*;
use lapin::types::FieldTable;
use lapin::{BasicProperties, Connection, ConnectionProperties};
use rustmq;
use std::env;

fn parse() -> String {
    let scheme = env::var("AMQP_SCHEME").unwrap_or_default();
    let user = env::var("AMQP_USERNAME").unwrap_or_default();
    let pass = env::var("AMQP_PASSWORD").unwrap_or_default();
    let cluster = env::var("AMQP_CLUSTER").unwrap_or_default();
    let vhost = env::var("AMQP_VHOST").unwrap_or_default();
    format!("{}://{}:{}@{}/{}", scheme, user, pass, cluster, vhost)
}

fn main() {
    let addr = parse();
    let con = Connection::connect(&addr, ConnectionProperties::default());
    let con = con.wait().expect("connection error");
    let a = con.create_channel().wait().expect("producer channel a");
    let b = con.create_channel().wait().expect("consumer channel b");
    let c = con.create_channel().wait().expect("consumer channel c");
    a.queue_declare(
        "hello",
        QueueDeclareOptions::default(),
        FieldTable::default(),
    )
    .wait()
    .expect("hello queue on a");
    let queue1 = b
        .queue_declare(
            "hello",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .wait()
        .expect("queue_declare");

    b.clone()
        .basic_consume(
            &queue1,
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .wait()
        .expect("basic_consume")
        .set_delegate(Box::new(rustmq::Consumer::new("b", b)));

    let queue = c
        .queue_declare(
            "hello",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .wait()
        .expect("queue_declare");
    c.clone()
        .basic_consume(
            &queue,
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .wait()
        .expect("basic_consume")
        .set_delegate(Box::new(rustmq::Consumer::new("c", c)));

    let payload = b"Hello world!";
    loop {
        a.basic_publish(
            "",
            "hello",
            BasicPublishOptions::default(),
            payload.to_vec(),
            BasicProperties::default(),
        )
        .wait()
        .expect("basic_publish");
    }
}
