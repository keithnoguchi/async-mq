# rustmq

[RabbitMQ] with [Crate lapin] for fun.

[![DroneCI]](https://cloud.drone.io/keithnoguchi/rustmq)
[![CircleCI]](https://circleci.com/gh/keithnoguchi/workflows/rustmq)

[DroneCI]: https://cloud.drone.io/api/badges/keithnoguchi/rustmq/status.svg
[CircleCI]: https://circleci.com/gh/keithnoguchi/rustmq.svg?style=svg

## Modules

- [client]: `Client` and `Connection` struct types
- [produce]: `Producer` trait type and the sample `Producer` trait instance
- [consume]: `Consumer` trait type and the sample `Consumer` trait instance
- [publish]: `Publisher` and `PublisherBuilder` struct types
- [subscribe]: `Subscriber` and `SubscriberBuilder` struct types
- [msg]: [Flatbuffers] based example messages

[client]: src/client.rs
[produce]: src/produce.rs
[consume]: src/consume.rs
[publish]: src/publish.rs
[subscribe]: src/subscribe.rs
[msg]: src/msg.rs
[main.rs]: src/main.rs
[flatbuffers]: https://google.github.io/flatbuffers/

## Example

Currently, [main.rs] demonstrates the [crate lapin] RabbitMQ RPC pattern
with the Rust 3.39 [async/.await] feature.  It creates 8 consumer threads
with 4 each consumer instance, hence 32 consumers, with 4 producer threads.
It uses [flatbuffers] to encode the message over the message bus and
producers just echo back the message to the consumer through the callback
queue.  Each producers sends alphabet 'a' to 'z' and dumps the result
to the stdout.  Here is the main function, which creates all those threads.

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

- [RabbitMQ]: The most widely deployed open source message broker
- [Crate lapin]: RabbitMQ crate based on AMQP 0.9.1 specification
  - [Lapin futures v0.3 example]: [Crate futures v0.3] example
- [The Async Book]: Asynchronous Programming in Rust
- [Crate futures]:
  - [Crate futures v0.3]: Abstructions for Asynchronous Programming
- [Original futures design]: Original futures design by [Aaron Turon]

[RabbitMQ]: https://www.rabbitmq.com
[crate lapin]: https://docs.rs/lapin/0.28.2/lapin/
[lapin futures v0.3 example]: https://github.com/sozu-proxy/lapin/blob/master/examples/pubsub_futures.rs
[crate lapin-futures]: https://docs.rs/lapin-futures/0.28.2/lapin_futures/
[the async book]: https://rust-lang.github.io/async-book/
[crate futures]: http://futures.rs/
[crate futures v0.3]: https://docs.rs/futures/0.3.1/
[original futures design]: https://aturon.github.io/blog/2016/09/07/futures-design/
[Aaron Turon]: https://aturon.github.io/blog/
