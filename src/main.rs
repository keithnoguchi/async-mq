// SPDX-License-Identifier: GPL-2.0
use futures_executor::LocalPool;
use futures_util::{future::FutureExt, stream::StreamExt, task::LocalSpawnExt};
use lapin::options::*;
use lapin::types::FieldTable;
use lapin::{BasicProperties, ConnectionProperties, Result};
use rustmq::Connection;
use std::env;

fn parse() -> String {
    let scheme = env::var("AMQP_SCHEME").unwrap_or_default();
    let user = env::var("AMQP_USERNAME").unwrap_or_default();
    let pass = env::var("AMQP_PASSWORD").unwrap_or_default();
    let cluster = env::var("AMQP_CLUSTER").unwrap_or_default();
    let vhost = env::var("AMQP_VHOST").unwrap_or_default();
    format!("{}://{}:{}@{}/{}", scheme, user, pass, cluster, vhost)
}

fn main() -> Result<()> {
    let mut exec = LocalPool::new();
    let spawner = exec.spawner();

    exec.run_until(async {
        let addr = parse();
        let con = Connection::connect(&addr, ConnectionProperties::default()).await?;

        // Consumers.
        for i in { b'a'..b'z' } {
            let c = con.create_channel().await?;
            let queue = c
                .queue_declare(
                    "hello",
                    QueueDeclareOptions::default(),
                    FieldTable::default(),
                )
                .await?;
            let consumer = c
                .clone()
                .basic_consume(
                    &queue,
                    "my_consumer",
                    BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await?;
            let x: char = i.into();
            let _consumer = spawner.spawn_local(async move {
                consumer
                    .for_each(move |delivery| {
                        eprint!("{}", x);
                        let delivery = delivery.expect("error caught in consumer");
                        c.basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                            .map(|_| ())
                    })
                    .await
            });
        }
        // Producer.
        let payload = b"Hello world!";
        let p = con.create_channel().await?;
        p.queue_declare(
            "hello",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
        loop {
            p.basic_publish(
                "",
                "hello",
                BasicPublishOptions::default(),
                payload.to_vec(),
                BasicProperties::default(),
            )
            .await?;
        }
    })
}
