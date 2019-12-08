# rustmq

[RabbitMQ] with [Crate lapin] for fun.

[![DroneCI]](https://cloud.drone.io/keithnoguchi/rustmq)
[![CircleCI]](https://circleci.com/gh/keithnoguchi/workflows/rustmq)

[DroneCI]: https://cloud.drone.io/api/badges/keithnoguchi/rustmq/status.svg
[CircleCI]: https://circleci.com/gh/keithnoguchi/rustmq.svg?style=svg

## Modules

- [client]: `Client` and `Connection` struct types
- [produce]: `Producer` trait type and the sample `Producer` trait instance
- [consume]: `Consumer, `ConsumerBuilder` and `ConsumerExt` trait
- [publish]: `Publisher` and `PublisherBuilder` struct types

[client]: src/client.rs
[produce]: src/produce.rs
[consume]: src/consume.rs
[publish]: src/publish.rs
[flatbuffers]: https://google.github.io/flatbuffers/

## Example

Currently, [main.rs] demonstrates the [crate lapin] RabbitMQ RPC pattern
with the Rust 1.39 [async/.await] feature.  It creates 32 producer threads
and 8 consumer threads, with each thread runs 8 consumer [async/.await]
instances.  It also uses [FlatBuffers] as a message encoding technology.
Currently, it just generate simple message with a single string, from
'a' to 'z'.

[main.rs]: src/main.rs
[async/.await]: https://blog.rust-lang.org/2019/11/07/Async-await-stable.html

Here is the current main function:

```sh
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
```

## Execution

Here is the output of the make run, e.g. cargo run.  It producers dumps
the messages echoed back by the consumers.

```sh
air2$ make run
   Compiling rustmq v0.3.0 (/home/kei/git/rustmq)
    Finished dev [unoptimized + debuginfo] target(s) in 1.88s
     Running `target/debug/rustmq`
aabaabbcbccdcddeefdeffgegghfhhigjiihkjjilkkjlmklmnmlnmonoponpqpoqqrrprssqtstruustvvtuwwuvxxvwyywxaxayybb
aacbcbdcdcededeffegffghgghihihijjijkkjklllkmmlnnmmoonnpopoqpqprqrqsrrstssttutuuvuvvwvwwwxxxxyyyyaaababbc
bcccddddeeeeffffgggghhhhiiiijjjjkklkkllmlmmnmnnonoopoppqpqqrqrsrrstsstuttvuuuwvvvxwwwyxxxayyybaaacbbbdcc
cedddfeeegfffgghghhihiijjikjkkljllkmmmnnnlooompppnqqqorrrpsssqtturtuvsuvwtxvwuyxwvayxwbayxcbadycbdaecebf
dfgceghdfhiegijfhjkigklhljmmkinlnojmopnkpqloqrmpsqnrortspsutqtvruwusvvxwtywuxaxvywbyaxcabydbcacdebdfeceg
fdfhgegihhjfikigjjlhkkmilnljmomkpnnlqoomprpnsqqotrrpussqvttruwusvxtvwyuwxvaxywbyaxcabydcbadcebedfecfg
^C
```

## References

- [The Async Book]: The Asynchronous Programming in Rust
  - [Async in traits], [crate async-trait], and [why async fn in traits are hard]
- [The Style Book]: The Rust Style Guidelines
- [RabbitMQ]: The most widely deployed open source message broker
- [Crate lapin]: RabbitMQ crate based on AMQP 0.9.1 specification
  - [Lapin futures v0.3 example]: [Crate futures v0.3] example
- [Crate futures v0.3]: Abstructions for Asynchronous Programming
- [Original futures design]: Original futures design by [Aaron Turon]

[the async book]: https://rust-lang.github.io/async-book/
[async in trait]: https://rust-lang.github.io/async-book/07_workarounds/06_async_in_traits.html
[crate async-trait]: https://github.com/dtolnay/async-trait
[why async fn in traits are hard]: https://smallcultfollowing.com/babysteps/blog/2019/10/26/async-fn-in-traits-are-hard/
[the style book]: https://doc.rust-lang.org/1.0.0/style/README.html
[RabbitMQ]: https://www.rabbitmq.com
[crate lapin]: https://docs.rs/lapin/0.28.2/lapin/
[lapin futures v0.3 example]: https://github.com/sozu-proxy/lapin/blob/master/examples/pubsub_futures.rs
[crate lapin-futures]: https://docs.rs/lapin-futures/0.28.2/lapin_futures/
[crate futures v0.3]: https://docs.rs/futures/0.3.1/
[original futures design]: https://aturon.github.io/blog/2016/09/07/futures-design/
[Aaron Turon]: https://aturon.github.io/blog/
