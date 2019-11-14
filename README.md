# rustmq

[RabbitMQ] with [Crate lapin-futures] and [Crate tokio] for fun.

[![DroneCI]](https://cloud.drone.io/keithnoguchi/rustmq)
[![CircleCI]](https://circleci.com/gh/keithnoguchi/workflows/rustmq)

[DroneCI]: https://cloud.drone.io/api/badges/keithnoguchi/rustmq/status.svg
[CircleCI]: https://circleci.com/gh/keithnoguchi/rustmq.svg?style=svg

## Modules

- [basic]: [Basic futures] example
- [peer]: [Getting asynchronous] example
- [combinator]: [Combinators] example
- [fibonacci]: [Streams] example
- [spawn]: [Spawning] example

[basic]: src/basic.rs
[peer]: src/peer.rs
[combinator]: src/combinator.rs
[fibonacci]: src/fibonacci.rs
[spawn]: src/spawn.rs
[basic futures]: https://tokio.rs/docs/futures/basic/
[getting asynchronous]: https://tokio.rs/docs/futures/getting_asynchronous/
[combinators]: https://tokio.rs/docs/futures/combinators/
[streams]: https://tokio.rs/docs/futures/streams/
[spawning]: https://tokio.rs/docs/futures/spawning/

## Execution

Currently, [main.rs] is the demonstration of [tokio getting-started] guide
to get familier with [tokio runtime].  Here is the snippet of `make run`,
which is a wrapper of `cargo run` as in [Makefile]:

```sh
air1$ make run
   Compiling rustmq v0.1.0 (/home/kei/git/rustmq)
    Finished dev [unoptimized + debuginfo] target(s) in 0.82s
     Running `target/debug/rustmq`
[server]: server running on 127.0.0.1:6142
[client]: About to create the stream and write to it...
[server]: connection from TcpStream { addr: V4(127.0.0.1:6142), pee
r: V4(127.0.0.1:50692), fd: 54 }
[peer::GetPeerAddr]: peer address = 127.0.0.1:6142
[server]: connection from TcpStream { addr: V4(127.0.0.1:6142), pee
r: V4(127.0.0.1:50694), fd: 5 }
created stream
[client]: wrote to stream; success=true
[server]: error: Connection reset by peer (os error 104)
[client]: Stream has been created and written to.
[server]: wrote 0 bytes
[basic::Display]: hello world
[basic::BetterDisplay]: hello world
^C
```

[main.rs]: src/main.rs
[Makefile]: Makefile

## References

- [RabbitMQ]: The most widely deployed open source message broker
- [Crate lapin-futures]: [Crate futures v0.1] based [Crate lapin]
- [Crate lapin]: RabbitMQ crate based on AMQP 0.9.1 specification
- [Crate tokio]: The asynchronous run-time for the Rust Programming Language
  - [Crate tokio v0.2]: A runtime for writing reliable, asynchronous, and slim applications
  - [Crate tokio v0.1]: An event-driven, non-blocking I/O platform for writing asynchronous I/O backed applications
- [Crate futures]:
  - [Crate futures v0.3]: Abstructions for Asynchronous Programming
  - [Crate futures v0.1]: Zero-cost Futures in Rust
- [Original futures design]: Original futures design by [Aaron Turon]

[RabbitMQ]: https://www.rabbitmq.com
[crate lapin-futures]: https://docs.rs/lapin-futures/0.28.2/lapin_futures/
[crate lapin]: https://docs.rs/lapin/0.28.2/lapin/
[crate tokio]: https://tokio.rs/
[crate tokio v0.2]: https://docs.rs/tokio/0.2.0-alpha.6/tokio/
[crate tokio v0.1]: https://docs.rs/crate/tokio/0.1.22
[tokio getting-started]: https://tokio.rs/docs/getting-started/hello-world/
[tokio runtime]: https://tokio.rs/docs/getting-started/runtime/
[crate futures]: http://futures.rs/
[crate futures v0.3]: https://docs.rs/futures/0.3.1/
[crate futures v0.1]: https://docs.rs/futures/0.1.29/
[original futures design]: https://aturon.github.io/blog/2016/09/07/futures-design/
[Aaron Turon]: https://aturon.github.io/blog/
