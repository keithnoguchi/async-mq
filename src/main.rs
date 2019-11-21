// SPDX-License-Identifier: GPL-2.0
use futures_executor::{LocalPool, LocalSpawner};
use futures_util::{future::FutureExt, stream::StreamExt, task::LocalSpawnExt};
use lapin::{options::*, Result};
use rustmq::{Consumer, Producer};
use std::{env, thread};

fn main() -> thread::Result<()> {
    let p = thread::spawn(|| {
        producer().expect("cannot start producer");
    });
    let c = thread::spawn(|| {
        let mut pool = LocalPool::new();
        let spawner = pool.spawner();
        spawner
            .spawn_local(consumers(spawner.clone()))
            .expect("cannot spawn consumers");
        pool.run()
    });
    p.join().expect("cannot join producer");
    c.join()
}

fn producer() -> Result<()> {
    let payload = b"Hello world!";
    let queue_name = "hello";
    let uri = parse();
    let mut pool = LocalPool::new();
    pool.run_until(async move {
        let mut producer = Producer::new(&uri, queue_name).await?;
        loop {
            producer.publish(payload.to_vec()).await?;
        }
    })
}

async fn consumers(spawner: LocalSpawner) {
    let queue_name = "hello";
    let uri = parse();
    let mut consumer = Consumer::new(&uri).await.expect("cannot create consumer");
    for i in { b'a'..b'z' } {
        let (worker, channel) = consumer
            .worker(queue_name)
            .await
            .expect("cannot create worker");
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
}

fn parse() -> String {
    let scheme = env::var("AMQP_SCHEME").unwrap_or_default();
    let user = env::var("AMQP_USERNAME").unwrap_or_default();
    let pass = env::var("AMQP_PASSWORD").unwrap_or_default();
    let cluster = env::var("AMQP_CLUSTER").unwrap_or_default();
    let vhost = env::var("AMQP_VHOST").unwrap_or_default();
    format!("{}://{}:{}@{}/{}", scheme, user, pass, cluster, vhost)
}
