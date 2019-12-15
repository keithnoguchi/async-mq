// SPDX-License-Identifier: APACHE-2.0 AND MIT
use flatbuffers::FlatBufferBuilder;
use futures_executor::{block_on, LocalPool};
use futures_util::{stream::StreamExt, task::LocalSpawnExt};
use rustmq::{prelude::*, Error};
use std::{env, thread};

fn main() -> thread::Result<()> {
    let mut threads = Vec::with_capacity(PRODUCER_THREAD_NR + CONSUMER_THREAD_NR);
    let client = Client::new();
    let request_queue = "request";
    let uri = parse();

    // A single connection for multiple local pool producers.
    let conn = block_on(client.connect(&uri)).expect("fail to connect");
    let mut builder = conn.producer_builder();
    builder.queue(String::from(request_queue));
    for _ in 0..PRODUCER_THREAD_NR {
        let builder = builder.clone();
        let producer = thread::spawn(move || {
            LocalPool::new().run_until(async {
                match builder.build().await {
                    Err(e) => eprintln!("{}", e),
                    Ok(p) => {
                        let mut p = ASCIIGenerator(p);
                        if let Err(err) = p.run().await {
                            eprintln!("{}", err);
                        }
                    }
                }
            });
        });
        threads.push(producer);
    }

    // A single connection for multiple local pool consumers.
    let conn = block_on(client.connect(&uri)).expect("fail to connect");
    let mut builder = conn.consumer_builder();
    builder.queue(String::from(request_queue));
    for _ in 0..CONSUMER_THREAD_NR {
        let builder = builder.clone();
        let consumer = thread::spawn(move || {
            let mut pool = LocalPool::new();
            let spawner = pool.spawner();
            for _ in 0..CONSUMER_INSTANCE_NR {
                let builder = builder.clone();
                spawner
                    .spawn_local(async move {
                        match builder.build().await {
                            Err(err) => eprintln!("{}", err),
                            Ok(c) => {
                                let mut c = EchoConsumer(c);
                                if let Err(err) = c.run().await {
                                    eprintln!("{}", err);
                                }
                            }
                        }
                    })
                    .expect("consumer died");
            }
            pool.run();
        });
        threads.push(consumer);
    }

    // Cleanup all instances.
    for t in threads {
        t.join()?;
    }
    Ok(())
}

struct ASCIIGenerator(Producer);

impl ASCIIGenerator {
    async fn run(&mut self) -> Result<(), Error> {
        let mut builder = FlatBufferBuilder::new();
        loop {
            // Generate ASCII character FlatBuffer messages
            // and print the received message to stderr.
            for data in { b'!'..=b'~' } {
                let req = Self::make_buf(&mut builder, vec![data]);
                let resp = self.0.rpc(req).await?;
                Self::print_buf(resp);
            }
        }
    }
    fn make_buf(builder: &mut FlatBufferBuilder, data: Vec<u8>) -> Vec<u8> {
        let data = builder.create_string(&String::from_utf8(data).unwrap());
        let mut mb = crate::msg::MessageBuilder::new(builder);
        mb.add_msg(data);
        let msg = mb.finish();
        builder.finish(msg, None);
        let req = builder.finished_data().to_vec();
        builder.reset();
        req
    }
    fn print_buf(resp: Vec<u8>) {
        if resp.is_empty() {
            return;
        }
        let msg = crate::msg::get_root_as_message(&resp);
        if let Some(data) = msg.msg() {
            eprint!("{}", data);
        }
    }
}

struct EchoConsumer(Consumer);

impl EchoConsumer {
    async fn run(&mut self) -> Result<(), Error> {
        while let Some(msg) = self.0.next().await {
            match msg {
                // Echo back the message.
                Ok(req) => self.0.response(&req, req.data()).await?,
                Err(err) => return Err(err),
            }
        }
        Ok(())
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
