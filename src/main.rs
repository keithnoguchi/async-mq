// SPDX-License-Identifier: APACHE-2.0 AND MIT
use async_trait::async_trait;
use flatbuffers::FlatBufferBuilder;
use futures_executor::{block_on, LocalPool, LocalSpawner};
use futures_util::task::LocalSpawnExt;
use lapin::Result;
use rustmq::{Client, PublisherBuilder, SubscriberBuilder};
use std::{env, thread};

#[derive(Clone)]
pub struct FlatBufferEchoProducer;

#[async_trait]
impl rustmq::Producer for FlatBufferEchoProducer {
    async fn recv(&mut self, msg: Vec<u8>) -> lapin::Result<()> {
        let msg = crate::msg::get_root_as_message(&msg);
        if let Some(msg) = msg.msg() {
            eprint!("{}", msg);
        }
        Ok(())
    }
    fn box_clone(&self) -> Box<dyn rustmq::Producer + Send> {
        Box::new((*self).clone())
    }
}

#[derive(Clone)]
struct EchoConsumer;

#[async_trait]
impl rustmq::Consumer for EchoConsumer {
    async fn consume(&mut self, msg: Vec<u8>) -> lapin::Result<Vec<u8>> {
        Ok(msg)
    }
    fn box_clone(&self) -> Box<dyn rustmq::Consumer + Send> {
        Box::new((*self).clone())
    }
}

fn main() -> thread::Result<()> {
    let client = Client::new();
    let uri = parse();
    let queue_name = "hello";

    // A single connection for the multiple consumers.
    let conn = block_on(client.connect(&uri)).expect("fail to connect");
    let mut builder = SubscriberBuilder::new(conn);
    builder.queue(String::from(queue_name));
    let mut consumers = Vec::with_capacity(8);
    for _ in 0..consumers.capacity() {
        let builder = builder.clone();
        let consumer = thread::spawn(move || {
            let mut pool = LocalPool::new();
            let spawner = pool.spawner();
            spawner
                .spawn_local(consumer(builder, spawner.clone()))
                .expect("consumers died");
            pool.run()
        });
        consumers.push(consumer);
    }

    // A single connection for the multiple producers.
    let conn = block_on(client.connect(&uri)).expect("fail to connect");
    let mut builder = PublisherBuilder::new(conn);
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

fn producer(builder: PublisherBuilder) -> Result<()> {
    let mut pool = LocalPool::new();
    pool.run_until(async move {
        let mut buf_builder = FlatBufferBuilder::new();
        let mut p = builder.build().await?;
        p.with_producer(Box::new(FlatBufferEchoProducer {}));
        loop {
            for data in { b'a'..b'z' } {
                let data = buf_builder.create_string(&String::from_utf8(vec![data]).unwrap());
                let mut mb = crate::msg::MessageBuilder::new(&mut buf_builder);
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

async fn consumer(builder: SubscriberBuilder, spawner: LocalSpawner) {
    for _ in 0usize..4 {
        let mut c = builder.build().await.expect("fail to build subscriber");
        let _task = spawner.spawn_local(async move {
            c.with_consumer(Box::new(EchoConsumer {}))
                .run()
                .await
                .expect("consumer died");
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

mod msg {
    #![allow(
        unused_imports,
        clippy::extra_unused_lifetimes,
        clippy::needless_lifetimes,
        clippy::redundant_closure,
        clippy::redundant_static_lifetimes
    )]
    include!("../schema/model_generated.rs");

    pub use model::get_root_as_message;
    pub use model::{Message, MessageArgs, MessageBuilder, MessageType};

    #[cfg(test)]
    mod tests {
        use flatbuffers::FlatBufferBuilder;
        #[test]
        fn message_create() {
            use super::get_root_as_message;
            use super::{Message, MessageArgs, MessageType};
            let msgs = ["a", "b", "c", "d"];
            for msg in &msgs {
                let mut b = FlatBufferBuilder::new();
                let bmsg = b.create_string(msg);
                let data = Message::create(
                    &mut b,
                    &MessageArgs {
                        msg: Some(bmsg),
                        ..Default::default()
                    },
                );
                b.finish(data, None);
                let buf = b.finished_data();
                let got = get_root_as_message(buf);
                assert_eq!(msg, &got.msg().unwrap());
                assert_eq!(0, got.id());
                assert_eq!(MessageType::Hello, got.msg_type());
                println!("mesg = {:?}", got);
            }
        }
        #[test]
        fn message_builder() {
            use super::get_root_as_message;
            use super::MessageType;
            let mut b = FlatBufferBuilder::new();
            let bmsg = b.create_string("a");
            let mut mb = super::MessageBuilder::new(&mut b);
            mb.add_id(1000);
            mb.add_msg(bmsg);
            mb.add_msg_type(super::MessageType::Goodbye);
            let data = mb.finish();
            b.finish(data, None);
            let buf = b.finished_data();
            let got = get_root_as_message(buf);
            assert_eq!("a", got.msg().unwrap());
            assert_eq!(1000, got.id());
            assert_eq!(MessageType::Goodbye, got.msg_type());
            println!("msg = {:?}", got);
        }
    }
}
