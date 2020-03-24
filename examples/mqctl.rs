// SPDX-License-Identifier: Apache-2.0 AND MIT
use async_mq::{prelude::*, Error};
use flatbuffers::FlatBufferBuilder;
use futures_util::stream::StreamExt;

pub enum Runtime {
    TokioThreaded,
    ThreadPool,
    LocalPool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = crate::cfg::Config::parse();
    match cfg.runtime {
        Runtime::TokioThreaded => tokio_threaded(cfg),
        Runtime::ThreadPool => thread_pool(cfg),
        Runtime::LocalPool => local_pool(cfg),
    }
}

fn tokio_threaded(cfg: crate::cfg::Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut rt = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_time()
        .build()?;
    let client = Client::new();

    rt.block_on(async move {
        // One connection for multiple producers.
        let conn = client.connect(&cfg.uri).await?;
        let mut builder = conn.producer_builder();
        builder.exchange(&cfg.exchange).queue(&cfg.queue);
        for _ in 0..cfg.producers {
            let builder = builder.clone();
            tokio::spawn(async move {
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
        }
        // One connection for multiple consumers.
        let conn = client.connect(&cfg.uri).await?;
        let mut builder = conn.consumer_builder();
        builder.exchange(&cfg.exchange).queue(&cfg.queue);
        for _ in 0..cfg.consumers {
            let builder = builder.clone();
            tokio::spawn(async move {
                match builder.build().await {
                    Err(err) => eprintln!("{}", err),
                    Ok(c) => {
                        let mut c = EchoConsumer(c);
                        if let Err(err) = c.run().await {
                            eprintln!("{}", err);
                        }
                    }
                }
            });
        }
        // idle loop.
        loop {
            tokio::time::delay_for(std::time::Duration::from_millis(1000)).await;
        }
    })
}

fn thread_pool(cfg: crate::cfg::Config) -> Result<(), Box<dyn std::error::Error>> {
    use futures::executor::block_on;
    use futures_executor::{enter, ThreadPool};
    use futures_util::task::SpawnExt;
    use std::thread;

    let pool = ThreadPool::new()?;
    let client = Client::new();

    // One connection for multiple thread pool producers and consumers each.
    let producer_conn = block_on(client.connect(&cfg.uri))?;
    let consumer_conn = block_on(client.connect(&cfg.uri))?;

    let enter = enter()?;
    let mut builder = producer_conn.producer_builder();
    builder.exchange(&cfg.exchange).queue(&cfg.queue);
    for _ in 0..cfg.producers {
        let builder = builder.clone();
        pool.spawn(async move {
            match builder.build().await {
                Err(e) => eprintln!("{}", e),
                Ok(p) => {
                    let mut p = ASCIIGenerator(p);
                    if let Err(err) = p.run().await {
                        eprintln!("{}", err);
                    }
                }
            }
        })?;
    }
    let mut builder = consumer_conn.consumer_builder();
    builder.exchange(&cfg.exchange).queue(&cfg.queue);
    for _ in 0..cfg.consumers {
        let builder = builder.clone();
        pool.spawn(async move {
            match builder.build().await {
                Err(err) => eprintln!("{}", err),
                Ok(c) => {
                    let mut c = EchoConsumer(c);
                    if let Err(err) = c.run().await {
                        eprintln!("{}", err);
                    }
                }
            }
        })?;
    }
    drop(enter);

    // idle loop.
    loop {
        thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn local_pool(cfg: crate::cfg::Config) -> Result<(), Box<dyn std::error::Error>> {
    use futures::executor::{block_on, LocalPool};
    use futures_util::task::LocalSpawnExt;
    use std::thread;

    let mut threads = Vec::new();
    let client = Client::new();

    // A single connection for multiple local pool producers.
    let conn = block_on(client.connect(&cfg.uri))?;
    let mut builder = conn.producer_builder();
    builder.exchange(&cfg.exchange).queue(&cfg.queue);
    for _ in 0..cfg.producers {
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
    let consumers_per_thread = cfg.consumers_per_thread;
    let consumers = cfg.consumers / consumers_per_thread;
    let conn = block_on(client.connect(&cfg.uri))?;
    let mut builder = conn.consumer_builder();
    builder.exchange(&cfg.exchange).queue(&cfg.queue);
    for _ in 0..consumers {
        let builder = builder.clone();
        let consumer = thread::spawn(move || {
            let mut pool = LocalPool::new();
            let spawner = pool.spawner();
            for _ in 0..consumers_per_thread {
                let builder = builder.clone();
                if let Err(err) = spawner.spawn_local(async move {
                    match builder.build().await {
                        Err(err) => eprintln!("{}", err),
                        Ok(c) => {
                            let mut c = EchoConsumer(c);
                            if let Err(err) = c.run().await {
                                eprintln!("{}", err);
                            }
                        }
                    }
                }) {
                    eprintln!("{:?}", err);
                }
            }
            pool.run();
        });
        threads.push(consumer);
    }

    // Cleanup all instances.
    for t in threads {
        if let Err(err) = t.join() {
            eprintln!("{:?}", err);
        }
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

mod cfg {
    const PRODUCERS: usize = 32;
    const CONSUMERS: usize = 64;
    const CONSUMERS_PER_THREAD: usize = 8;

    pub struct Config {
        pub uri: String,
        pub exchange: String,
        pub queue: String,
        pub runtime: super::Runtime,
        pub producers: usize,
        pub consumers: usize,
        pub consumers_per_thread: usize,
    }

    impl std::str::FromStr for super::Runtime {
        type Err = std::string::ParseError;
        fn from_str(name: &str) -> Result<Self, Self::Err> {
            if name.starts_with("tokio") {
                Ok(Self::TokioThreaded)
            } else if name.starts_with("thread") {
                Ok(Self::ThreadPool)
            } else {
                Ok(Self::LocalPool)
            }
        }
    }

    impl Config {
        pub fn parse() -> Self {
            use clap::{value_t, App, Arg, SubCommand};
            let producers = PRODUCERS.to_string();
            let consumers = CONSUMERS.to_string();
            let consumers_per_thread = CONSUMERS_PER_THREAD.to_string();
            let opts = App::new(env!("CARGO_PKG_DESCRIPTION"))
                .author(env!("CARGO_PKG_AUTHORS"))
                .arg(
                    Arg::with_name("runtime")
                        .short("r")
                        .long("runtime")
                        .help("Rust runtime")
                        .takes_value(true)
                        .default_value("tokio")
                        .possible_values(&["tokio", "thread-pool", "local-pool"]),
                )
                .arg(
                    Arg::with_name("username")
                        .short("u")
                        .long("username")
                        .help("AMQP username")
                        .takes_value(true)
                        .default_value("rabbit"),
                )
                .arg(
                    Arg::with_name("password")
                        .short("p")
                        .long("password")
                        .help("AMQP password")
                        .takes_value(true)
                        .default_value("RabbitMQ"),
                )
                .arg(
                    Arg::with_name("scheme")
                        .short("s")
                        .long("scheme")
                        .help("AMQP scheme")
                        .takes_value(true)
                        .default_value("amqp")
                        .possible_values(&["amqp", "amqps"]),
                )
                .arg(
                    Arg::with_name("cluster")
                        .short("c")
                        .long("cluster")
                        .help("AMQP cluster")
                        .takes_value(true)
                        .default_value("127.0.0.1:5672"),
                )
                .arg(
                    Arg::with_name("vhost")
                        .short("v")
                        .long("vhost")
                        .help("AMQP vhost name")
                        .takes_value(true)
                        .default_value("mx"),
                )
                .arg(
                    Arg::with_name("exchange")
                        .short("x")
                        .long("exchange")
                        .help("AMQP exchange name")
                        .takes_value(true)
                        .default_value("async-mq"),
                )
                .arg(
                    Arg::with_name("queue")
                        .short("q")
                        .long("queue")
                        .help("AMQP queue name")
                        .takes_value(true)
                        .default_value("request"),
                )
                .subcommand(
                    SubCommand::with_name("tune")
                        .about("Tuning parameters")
                        .arg(
                            Arg::with_name("producers")
                                .short("p")
                                .long("producers")
                                .help("Number of producers")
                                .takes_value(true)
                                .default_value(&producers),
                        )
                        .arg(
                            Arg::with_name("consumers")
                                .short("c")
                                .long("consumers")
                                .help("Number of consumers")
                                .takes_value(true)
                                .default_value(&consumers),
                        )
                        .arg(
                            Arg::with_name("consumers-per-thread")
                                .short("t")
                                .long("consumers-per-thread")
                                .help("Number of consumers")
                                .takes_value(true)
                                .default_value(&consumers_per_thread),
                        ),
                )
                .get_matches();
            let runtime = value_t!(opts, "runtime", super::Runtime).unwrap();
            let scheme = opts.value_of("scheme").unwrap();
            let user = opts.value_of("username").unwrap();
            let pass = opts.value_of("password").unwrap();
            let cluster = opts.value_of("cluster").unwrap();
            let vhost = opts.value_of("vhost").unwrap();
            let uri = format!("{}://{}:{}@{}/{}", scheme, user, pass, cluster, vhost);
            let exchange = opts.value_of("exchange").unwrap();
            let queue = opts.value_of("queue").unwrap();
            let mut producers = PRODUCERS;
            let mut consumers = PRODUCERS;
            let mut consumers_per_thread = CONSUMERS_PER_THREAD;
            if let Some(opts) = opts.subcommand_matches("tune") {
                if let Ok(val) = value_t!(opts, "producers", usize) {
                    producers = val;
                }
                if let Ok(val) = value_t!(opts, "consumers", usize) {
                    consumers = val;
                }
                if let Ok(val) = value_t!(opts, "consumers_per_thread", usize) {
                    consumers_per_thread = val;
                }
            }
            Self {
                uri,
                exchange: exchange.to_string(),
                queue: queue.to_string(),
                runtime,
                producers,
                consumers,
                consumers_per_thread,
            }
        }
    }
}

mod msg {
    #![allow(
        unused_imports,
        clippy::extra_unused_lifetimes,
        clippy::needless_lifetimes,
        clippy::redundant_closure,
        clippy::redundant_static_lifetimes
    )]
    include!("./schema/model_generated.rs");

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
