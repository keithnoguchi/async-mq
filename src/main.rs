// SPDX-License-Identifier: GPL-2.0
use futures_executor::{LocalPool, LocalSpawner};
use futures_util::{future::FutureExt, stream::StreamExt, task::LocalSpawnExt};
use lapin::{options::*, Result};
use rustmq::Client;
use std::{env, thread};

fn main() -> thread::Result<()> {
    let p = thread::spawn(|| {
        let queue_name = "hello";
        let uri = parse();
        producer(uri, &queue_name).expect("cannot start producer");
    });
    let c = thread::spawn(|| {
        let queue_name = "hello";
        let uri = parse();
        let mut pool = LocalPool::new();
        let spawner = pool.spawner();
        spawner
            .spawn_local(consumers(uri.clone(), &queue_name, spawner.clone()))
            .expect("cannot spawn consumers");
        pool.run()
    });
    p.join().expect("cannot join producer");
    c.join()
}

fn producer(uri: String, queue_name: &'static str) -> Result<()> {
    let mut pool = LocalPool::new();
    let payload = b"Hello world!";
    pool.run_until(async move {
        let mut c = Client::new(&uri).await?;
        let mut producer = c.producer(queue_name).await?;
        loop {
            producer.publish(payload.to_vec()).await?;
        }
    })
}

async fn consumers(uri: String, queue_name: &'static str, spawner: LocalSpawner) {
    let mut c = Client::new(&uri).await.expect("cannot create client");
    let mut consumer = c.consumer().await.expect("cannot create consumer");
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
