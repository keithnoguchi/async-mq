// SPDX-License-Identifier: APACHE-2.0 AND MIT
use async_trait::async_trait;
use flatbuffers::FlatBufferBuilder;
use futures_executor::{block_on, LocalPool, LocalSpawner};
use futures_util::{stream::StreamExt, task::LocalSpawnExt};
use rustmq::prelude::*;
use std::{env, thread};

fn main() -> thread::Result<()> {
    let mut threads = Vec::with_capacity(PRODUCER_THREAD_NR + CONSUMER_THREAD_NR);
    let client = Client::new();
    let queue_name = "hello";
    let uri = parse();

    // A single connection for the multiple producers.
    let conn = block_on(client.connect(&uri)).expect("fail to connect");
    let mut builder = conn.producer_builder();
    builder.queue(String::from(queue_name));
    for _ in 0..PRODUCER_THREAD_NR {
        let builder = builder.clone();
        let producer = thread::spawn(move || {
            ASCIIGenerator::new(builder).run().expect("generator died");
        });
        threads.push(producer);
    }

    // A single connection for multiple consumers.
    let conn = block_on(client.connect(&uri)).expect("fail to connect");
    let mut builder = conn.consumer_builder();
    builder.queue(String::from(queue_name));
    for _ in 0..CONSUMER_THREAD_NR {
        let builder = builder.clone();
        let consumer = thread::spawn(move || {
            LocalEchoConsumer::new(builder, CONSUMER_INSTANCE_NR).run();
        });
        threads.push(consumer);
    }

    // Cleanup all instances.
    for t in threads {
        t.join()?;
    }
    Ok(())
}

struct ASCIIGenerator {
    builder: ProducerBuilder,
}

impl ASCIIGenerator {
    fn new(builder: ProducerBuilder) -> Self {
        Self { builder }
    }
    fn run(&mut self) -> Result<(), rustmq::Error> {
        let mut builder = self.builder.clone();
        builder.with_handler(Box::new(NoopPeeker {}));
        let mut pool = LocalPool::new();
        pool.run_until(async move {
            let mut producer = builder.build().await?;
            let mut builder = FlatBufferBuilder::new();
            loop {
                // Generate ASCII character FlatBuffer messages
                // and print the received message to stderr.
                for data in { b'!'..b'~' } {
                    let req = self.make_buf(&mut builder, vec![data]);
                    let resp = producer.rpc(req).await?;
                    self.print_buf(resp);
                }
            }
        })
    }
    fn make_buf(&self, builder: &mut FlatBufferBuilder, data: Vec<u8>) -> Vec<u8> {
        let data = builder.create_string(&String::from_utf8(data).unwrap());
        let mut mb = crate::msg::MessageBuilder::new(builder);
        mb.add_msg(data);
        let msg = mb.finish();
        builder.finish(msg, None);
        let req = builder.finished_data().to_vec();
        builder.reset();
        req
    }
    fn print_buf(&self, resp: Vec<u8>) {
        if resp.is_empty() {
            return;
        }
        let msg = crate::msg::get_root_as_message(&resp);
        if let Some(data) = msg.msg() {
            eprint!("{}", data);
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct NoopPeeker;

#[async_trait]
impl ProducerHandler for NoopPeeker {
    async fn peek(&mut self, msg: Vec<u8>) -> Result<Vec<u8>, rustmq::Error> {
        // Nothing to do now.
        Ok(msg)
    }
    fn boxed_clone(&self) -> Box<dyn ProducerHandler + Send> {
        Box::new((*self).clone())
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct DropPeeker;

#[async_trait]
impl ProducerHandler for DropPeeker {
    /// Just comsume the received message so that no message print out
    /// to the console.  This is good for the benchmarking.
    async fn peek(&mut self, _msg: Vec<u8>) -> Result<Vec<u8>, rustmq::Error> {
        Ok(vec![])
    }
    fn boxed_clone(&self) -> Box<dyn ProducerHandler + Send> {
        Box::new((*self).clone())
    }
}

struct LocalEchoConsumer {
    builder: ConsumerBuilder,
    consumers: usize,
    pool: LocalPool,
    spawner: LocalSpawner,
}

impl LocalEchoConsumer {
    fn new(builder: ConsumerBuilder, consumers: usize) -> Self {
        let pool = LocalPool::new();
        let spawner = pool.spawner();
        Self {
            builder,
            consumers,
            pool,
            spawner,
        }
    }
    fn run(mut self) {
        let builder = self.builder.clone();
        let spawner = self.spawner.clone();
        let consumers = self.consumers;
        self.spawner
            .spawn_local(async move {
                for _ in 0..consumers {
                    let mut c = builder.build().await.expect("consumer build failed");
                    let _task = spawner.spawn_local(async move {
                        while let Some(Ok(req)) = c.next().await {
                            // Echo back the message.
                            if let Err(err) = c.response(&req, req.data()).await {
                                eprintln!("{}", err);
                            }
                        }
                    });
                }
            })
            .expect("consumer manager died");
        self.pool.run();
    }
}

const TOTAL_PRODUCER_NR: usize = 32;
const TOTAL_CONSUMER_NR: usize = 64;
const PRODUCER_THREAD_NR: usize = TOTAL_PRODUCER_NR;
const CONSUMER_THREAD_NR: usize = 8;
const CONSUMER_INSTANCE_NR: usize = TOTAL_CONSUMER_NR / CONSUMER_THREAD_NR;

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
