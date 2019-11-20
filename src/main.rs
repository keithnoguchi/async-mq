// SPDX-License-Identifier: GPL-2.0
use lapin::message::DeliveryResult;
use lapin::options::*;
use lapin::types::FieldTable;
use lapin::ConsumerDelegate;
use lapin::{BasicProperties, Channel, Connection, ConnectionProperties};
use std::env;

#[derive(Clone, Debug)]
struct Subscriber {
    channel: Channel,
}

impl ConsumerDelegate for Subscriber {
    fn on_new_delivery(&self, delivery: DeliveryResult) {
        if let Some(delivery) = delivery.unwrap() {
            print!(".");
            self.channel
                .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                .wait()
                .expect("basic_ack")
        }
    }
}

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
    let c = Connection::connect(&addr, ConnectionProperties::default());
    let c = c.wait().expect("connection error");
    let a = c.create_channel().wait().expect("channel a");
    let b = c.create_channel().wait().expect("channel b");
    a.queue_declare(
        "hello",
        QueueDeclareOptions::default(),
        FieldTable::default(),
    )
    .wait()
    .expect("hello queue on a");
    let queue = b
        .queue_declare(
            "hello",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .wait()
        .expect("queue_declare");

    b.clone()
        .basic_consume(
            &queue,
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .wait()
        .expect("basic_consume")
        .set_delegate(Box::new(Subscriber { channel: b }));

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
