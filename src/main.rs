// SPDX-License-Identifier: GPL-2.0
use futures_executor::LocalPool;
use futures_util::{future::FutureExt, stream::StreamExt, task::LocalSpawnExt};
use lapin::{options::*, BasicProperties, Result};
use rustmq::{Consumer, Producer};
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
        let uri = parse();
        let mut consumer = Consumer::new(&uri).await?;
        // Consumers.
        for i in { b'a'..b'z' } {
            let (worker, channel) = consumer.worker("hello").await?;
            let x: char = i.into();
            let _task = spawner.spawn_local(async move {
                worker
                    .for_each(move |delivery| {
                        eprint!("{}", x);
                        let delivery = delivery.expect("error caught in consumer");
                        channel
                            .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                            .map(|_| ())
                    })
                    .await
            });
        }
        // Producer.
        let producer = Producer::new(&uri, "hello").await?;
        let payload = b"Hello world!";
        loop {
            producer
                .channel
                .basic_publish(
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
