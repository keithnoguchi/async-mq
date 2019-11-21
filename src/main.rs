// SPDX-License-Identifier: GPL-2.0
use futures_executor::LocalPool;
use futures_util::{future::FutureExt, stream::StreamExt, task::LocalSpawnExt};
use lapin::{options::*, Result};
use rustmq::{Consumer, Producer};
use std::env;

fn main() -> Result<()> {
    let mut pool = LocalPool::new();
    let spawner = pool.spawner();

    pool.run_until(async {
        let queue_name = "hello";
        let uri = parse();

        // Consumers.
        let mut consumer = Consumer::new(&uri).await?;
        for i in { b'a'..b'z' } {
            let (worker, channel) = consumer.worker(queue_name).await?;
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
        let mut producer = Producer::new(&uri, queue_name).await?;
        let payload = b"Hello world!";
        loop {
            producer.publish(payload.to_vec()).await?;
        }
    })
}

fn parse() -> String {
    let scheme = env::var("AMQP_SCHEME").unwrap_or_default();
    let user = env::var("AMQP_USERNAME").unwrap_or_default();
    let pass = env::var("AMQP_PASSWORD").unwrap_or_default();
    let cluster = env::var("AMQP_CLUSTER").unwrap_or_default();
    let vhost = env::var("AMQP_VHOST").unwrap_or_default();
    format!("{}://{}:{}@{}/{}", scheme, user, pass, cluster, vhost)
}
