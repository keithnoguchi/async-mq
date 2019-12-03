// SPDX-License-Identifier: GPL-2.0
use futures_executor::{block_on, LocalPool, LocalSpawner};
use futures_util::{future::FutureExt, stream::StreamExt, task::LocalSpawnExt};
use lapin::{options::*, Result};
use rustmq::Client;
use std::{env, thread};

fn main() -> thread::Result<()> {
    // Using a single client connection to the rabbit broker.
    let client1 = client(parse());
    let client2 = client1.clone();
    let p = thread::spawn(move || {
        let queue_name = "hello";
        producer(client1, &queue_name).expect("cannot start producer");
    });
    let c = thread::spawn(move || {
        let queue_name = "hello";
        let mut pool = LocalPool::new();
        let spawner = pool.spawner();
        spawner
            .spawn_local(consumers(client2, &queue_name, spawner.clone()))
            .expect("cannot spawn consumers");
        pool.run()
    });
    p.join().expect("cannot join producer");
    c.join()
}

fn client(uri: String) -> Client {
    let client = Client::new(&uri);
    block_on(client).unwrap()
}

fn producer(mut c: Client, queue_name: &'static str) -> Result<()> {
    let mut pool = LocalPool::new();
    let payload = b"Hello world!";
    pool.run_until(async move {
        let mut producer = c.producer(queue_name).await?;
        loop {
            producer.publish(payload.to_vec()).await?;
        }
    })
}

async fn consumers(mut c: Client, queue_name: &'static str, spawner: LocalSpawner) {
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
