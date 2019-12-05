// SPDX-License-Identifier: GPL-2.0
use flatbuffers::FlatBufferBuilder;
use futures_executor::{block_on, LocalPool, LocalSpawner};
use futures_util::{future::FutureExt, stream::StreamExt, task::LocalSpawnExt};
use lapin::{options::*, Result};
use rustmq::{Client, Consumer, Producer};
use std::{env, thread};

fn main() -> thread::Result<()> {
    // Using a single client connection to the rabbit broker.
    let url = parse();
    let client1 = Client::new(&url);
    let client1 = block_on(client1).unwrap();
    let client2 = client1.clone();
    let queue_name = "hello";
    let p = thread::spawn(move || {
        producer(client1, String::from(queue_name)).expect("cannot start producer");
    });
    let c = thread::spawn(move || {
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

fn producer(c: Client, queue_name: String) -> Result<()> {
    let mut pool = LocalPool::new();
    pool.run_until(async move {
        let mut builder = FlatBufferBuilder::new();
        let mut p = Producer::new(c, queue_name);
        loop {
            for data in { b'a'..b'z' } {
                let data = builder.create_string(&String::from_utf8(vec![data]).unwrap());
                let mut mb = rustmq::MessageBuilder::new(&mut builder);
                mb.add_msg(data);
                let msg = mb.finish();
                builder.finish(msg, None);
                let msg = builder.finished_data();
                p.publish(msg.to_vec()).await?;
                builder.reset();
            }
        }
    })
}

async fn consumers(c: Client, queue_name: &'static str, spawner: LocalSpawner) {
    let mut c = Consumer::new(c);
    for i in 0..4 {
        let (worker, channel) = c.worker(queue_name).await.expect("cannot create worker");
        let _task = spawner.spawn_local(async move {
            worker
                .for_each(move |delivery| {
                    let delivery = delivery.expect("error caught in consumer");
                    let msg = rustmq::get_root_as_message(&delivery.data);
                    print!("{}[{}]", i, msg.msg().unwrap());
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
