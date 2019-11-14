# rustmq

[RabbitMQ] with [Crate lapin-futures] and [Crate tokio] for fun.

[![DroneCI]](https://cloud.drone.io/keithnoguchi/rustmq)
[![CircleCI]](https://circleci.com/gh/keithnoguchi/workflows/rustmq)

[DroneCI]: https://cloud.drone.io/api/badges/keithnoguchi/rustmq/status.svg
[CircleCI]: https://circleci.com/gh/keithnoguchi/rustmq.svg?style=svg

## Modules

- [echo]: [Hello World] example
- [basic]: [Basic futures] example
- [peer]: [Getting asynchronous] example
- [combinator]: [Combinators] example
- [fibonacci]: [Streams] example
- [spawn]: [Spawning] example

[echo]: src/echo.rs
[basic]: src/basic.rs
[peer]: src/peer.rs
[combinator]: src/combinator.rs
[fibonacci]: src/fibonacci.rs
[spawn]: src/spawn.rs
[hello world]: https://tokio.rs/docs/getting-started/hello-world/
[basic futures]: https://tokio.rs/docs/futures/basic/
[getting asynchronous]: https://tokio.rs/docs/futures/getting_asynchronous/
[combinators]: https://tokio.rs/docs/futures/combinators/
[streams]: https://tokio.rs/docs/futures/streams/
[spawning]: https://tokio.rs/docs/futures/spawning/

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

Currently, [main.rs] is the demonstration of [tokio getting-started] guide
to get familier with [tokio runtime].  Here is the snippet of `make run`,
which is a wrapper of `cargo run` as in [Makefile]:

```sh
$ make run
   Compiling rustmq v0.1.0 (/home/kei/git/rustmq)
    Finished dev [unoptimized + debuginfo] target(s) in 0.83s
     Running `target/debug/rustmq`
[peer::HelloWorld]: poll()
[peer::HelloWorld]: poll()
[basic::Display]: hello world
[peer::HelloWorld]: Connecting
[peer::HelloWorld]: Connected
[basic::BetterDisplay]: hello world
[echo::server]: connection from TcpStream { addr: V4(127.0.0.1:6142), peer: V4(127.0.0.1:48716), fd: 21 }
[echo::server]: connection from TcpStream { addr: V4(127.0.0.1:6142), peer: V4(127.0.0.1:48718), fd: 16 }
[peer::GetPeerAddr[echo::server]: connection from TcpStream { addr: V4(127.0.0.1:6142), peer: V4(127.0.0.1:48720), fd: 22
}
]: NotReady
[combinator::HelloWorld] poll()
[echo::server]: connection from TcpStream { addr: V4(127.0.0.1:6142), peer: V4(127.0.0.1:48722), fd: 23 }
[combinator::hello]: [combinator::HelloWorld]: hello world
[echo::server]: connection from TcpStream { addr: V4(127.0.0.1:6142), peer: V4(127.0.0.1:48724), fd: 24 }
[echo::client_and_then]: write complete
[peer::GetPeerAddr]: peer address = 127.0.0.1:6142
[echo::server]: wrote 11 bytes
[echo::client]: created stream
[echo::client]: wrote to stream; success=true
[echo::server]: wrote 0 bytes
[echo::server]: wrote 11 bytes
[echo::server]: wrote 12 bytes
[echo::client_and_then_and_then]: got [104, 101, 108, 108, 111, 32, 119, 111, 114, 108]
[echo::server]: error: Connection reset by peer (os error 104)
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
- [Future by example]: Get start working with Rust's Future quickly
- [Tokio internals]: Understanding Rust's asynchronous I/O framework from the bottom up
- [Original futures design]: Original futures design by [Aaron Turon]

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
[tokio internals]: https://cafbit.com/post/tokio_internals/
[original futures design]: https://aturon.github.io/blog/2016/09/07/futures-design/
[Aaron Turon]: https://aturon.github.io/blog/
