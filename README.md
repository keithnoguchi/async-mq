# rustmq

[RabbitMQ] with [Crate lapin] for fun.

[![DroneCI]](https://cloud.drone.io/keithnoguchi/rustmq)
[![CircleCI]](https://circleci.com/gh/keithnoguchi/workflows/rustmq)

[DroneCI]: https://cloud.drone.io/api/badges/keithnoguchi/rustmq/status.svg
[CircleCI]: https://circleci.com/gh/keithnoguchi/rustmq.svg?style=svg

## Modules

- [Hello World!] examples
  - [hello]: [Hello World] example
- [Working with Futures] examples
  - [basic]: [Basic futures] example
  - [peer]: [Getting asynchronous] example
  - [combinator]: [Combinators] example
  - [fibonacci]: [Streams] example
  - [spawn]: [Spawning] example
- [I/O with Tokio]
  - [echo]: [I/O overview] example
- [Going Deeper]
  - [deeper]: [Implementing Future] example
- [Tokio Internals]

[hello]: src/hello.rs
[basic]: src/basic.rs
[peer]: src/peer.rs
[combinator]: src/combinator.rs
[fibonacci]: src/fibonacci.rs
[spawn]: src/spawn.rs
[echo]: src/echo.rs
[deeper]: src/deeper.rs
[hello world]: https://tokio.rs/docs/getting-started/hello-world/
[basic futures]: https://tokio.rs/docs/futures/basic/
[getting asynchronous]: https://tokio.rs/docs/futures/getting_asynchronous/
[combinators]: https://tokio.rs/docs/futures/combinators/
[streams]: https://tokio.rs/docs/futures/streams/
[spawning]: https://tokio.rs/docs/futures/spawning/
[i/o overview]: https://tokio.rs/docs/io/overview/
[implementing future]: https://tokio.rs/docs/going-deeper/futures/

## Test

Currently, there are ten unit test in some of the modules, though it's not
actually testing anything but just printing out the result.  I'll come back
later to make it actaully test the behavior.

```sh
$ make test | tail -16
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running target/debug/deps/rustmq-a5a15abce52d5c74
[combinator::HelloWorld] poll()
     Running target/debug/deps/rustmq-e666ab5ce730d6e4
   Doc-tests rustmq
value #48 = 7778742049
value #49 = 12586269025
test fibonacci::test::display_slow_fibonacci ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Execution

Currently, [main.rs] hosts the [crate lapin] example to show the producer
and consumer communication, as shown below:

```sh
air2$ make run
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/rustmq`
..................................................................................................
..................................................................................................
..................................................................................................
..................................................................................................
..................................................................................................
..................................................................................................
..................................................................................................
..................................................................................................
..................................................................................................
..................................................................................................
............................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................^C
make: *** [Makefile:11: run] Interrupt

```

[main.rs]: src/main.rs

## References

- [RabbitMQ]: The most widely deployed open source message broker
- [Crate lapin-futures]: [Crate futures v0.1] based [Crate lapin]
- [Crate lapin]: RabbitMQ crate based on AMQP 0.9.1 specification
- [Tokio Getting Started guide]
  - [Hello World!]
  - [Working with Futures]
  - [I/O with Tokio]
  - [Going Deeper]
  - [Tokio Internals]
- [Understanding Tokio internals]: Understanding Rust's asynchronous I/O framework from the bottom up
- [Crate tokio]: The asynchronous run-time for the Rust Programming Language
  - [Crate tokio v0.2]: A runtime for writing reliable, asynchronous, and slim applications
  - [Crate tokio v0.1]: An event-driven, non-blocking I/O platform for writing asynchronous I/O backed applications
- [Future by example]: Get start working with Rust's Future quickly
- [Original futures design]: Original futures design by [Aaron Turon]
- [Crate futures]:
  - [Crate futures v0.3]: Abstructions for Asynchronous Programming
  - [Crate futures v0.1]: Zero-cost Futures in Rust

[RabbitMQ]: https://www.rabbitmq.com
[crate lapin-futures]: https://docs.rs/lapin-futures/0.28.2/lapin_futures/
[crate lapin]: https://docs.rs/lapin/0.28.2/lapin/
[crate tokio]: https://tokio.rs/
[crate tokio v0.2]: https://docs.rs/tokio/0.2.0-alpha.6/tokio/
[crate tokio v0.1]: https://docs.rs/tokio/0.1.22/tokio/
[tokio getting-started]: https://tokio.rs/docs/getting-started/hello-world/
[tokio runtime]: https://tokio.rs/docs/getting-started/runtime/
[crate futures]: http://futures.rs/
[crate futures v0.3]: https://docs.rs/futures/0.3.1/
[crate futures v0.1]: https://docs.rs/futures/0.1.29/
[future by example]: https://docs.rs/future-by-example/0.1.0/future_by_example/
[tokio getting started guide]: https://tokio.rs/docs/overview/
[hello world!]: https://tokio.rs/docs/getting-started/hello-world/
[working with futures]: https://tokio.rs/docs/futures/overview/
[going deeper]: https://tokio.rs/docs/going-deeper/futures/
[i/o with tokio]: https://tokio.rs/docs/io/overview/
[tokio internals]: https://tokio.rs/docs/internals/intro/
[understanding tokio internals]: https://cafbit.com/post/tokio_internals/
[original futures design]: https://aturon.github.io/blog/2016/09/07/futures-design/
[Aaron Turon]: https://aturon.github.io/blog/
