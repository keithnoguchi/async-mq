// SPDX-License-Identifier: GPL-2.0
use flatbuffers::FlatBufferBuilder;
use futures_executor::{block_on, LocalPool, LocalSpawner};
use futures_util::task::LocalSpawnExt;
use lapin::Result;
use rustmq::{Client, ConsumerBuilder, ProducerBuilder};
use std::{env, thread};

fn main() -> thread::Result<()> {
    let client = Client::new();
    let uri = parse();
    let queue_name = "hello";

    // A single connection for the multiple consumers.
    let conn = block_on(client.connect(&uri)).expect("fail to connect");
    let mut consumers = Vec::with_capacity(8);
    for _ in 0..consumers.capacity() {
        let builder = ConsumerBuilder::new(conn.clone());
        let consumer = thread::spawn(move || {
            let mut pool = LocalPool::new();
            let spawner = pool.spawner();
            spawner
                .spawn_local(consumer(builder, &queue_name, spawner.clone()))
                .expect("consumers died");
            pool.run()
        });
        consumers.push(consumer);
    }

    // A single connection for the multiple producers.
    let conn = block_on(client.connect(&uri)).expect("fail to connect");
    let mut builder = ProducerBuilder::new(conn);
    builder.queue(String::from(queue_name));
    let mut producers = Vec::with_capacity(4);
    for _ in 0..producers.capacity() {
        let builder = builder.clone();
        let producer = thread::spawn(move || {
            producer(builder).expect("producer died");
        });
        producers.push(producer);
    }

    while !consumers.is_empty() {
        let consumer = consumers.pop().unwrap();
        consumer.join()?;
    }
    while !producers.is_empty() {
        let producer = producers.pop().unwrap();
        producer.join()?;
    }
    Ok(())
}

fn producer(builder: ProducerBuilder) -> Result<()> {
    let mut pool = LocalPool::new();
    pool.run_until(async move {
        let mut buf_builder = FlatBufferBuilder::new();
        let mut p = builder.build().await?;
        loop {
            for data in { b'a'..b'z' } {
                let data = buf_builder.create_string(&String::from_utf8(vec![data]).unwrap());
                let mut mb = rustmq::MessageBuilder::new(&mut buf_builder);
                mb.add_msg(data);
                let msg = mb.finish();
                buf_builder.finish(msg, None);
                let msg = buf_builder.finished_data();
                p.rpc(msg.to_vec()).await?;
                buf_builder.reset();
            }
        }
    })
}

async fn consumer(mut builder: ConsumerBuilder, queue_name: &'static str, spawner: LocalSpawner) {
    for _ in 0usize..4 {
        let mut consumer = builder.consumer(queue_name).await.expect("consumer died");
        let _task = spawner.spawn_local(async move {
            consumer.run().await.expect("consumer error");
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
