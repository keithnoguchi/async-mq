# rustmq

[RabbitMQ] with [Crate lapin] for fun.

[![DroneCI]](https://cloud.drone.io/keithnoguchi/rustmq)
[![CircleCI]](https://circleci.com/gh/keithnoguchi/workflows/rustmq)

[DroneCI]: https://cloud.drone.io/api/badges/keithnoguchi/rustmq/status.svg
[CircleCI]: https://circleci.com/gh/keithnoguchi/rustmq.svg?style=svg

## Modules

- [consume]: for Struct Consumer

[consume]: src/consume.rs

## Execution

Currently, [main.rs] demonstrates the [crate lapin] example with 26 consumers,
each dumps their own names over the stdout, to prove the multiple consumers
over single AMQP connection works.

```sh
air2$ make run
    Finished dev [unoptimized + debuginfo] target(s) in 0.07s
     Running `target/debug/rustmq`
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaagacbdefhijkl
mnopqrsbtdefguhvjkwxycilamnopqrsutwvxyabcdefghijklmnopqrstuvwxyabcdefghijklcmnopaqbrstuvdwxyefghij
kmnopqrstxluvwdefyghibajcklmnopqrstuvwxybacdefghijklmnorstuvwxyabcdefghijklmnorstuvwxyabcdefghijkl
mnorstuvwxyabcdefghijppppklmnoqqqqrstvuwxyabcgdefhijklmnoqprstuvwxyabcdgefihjlkmnopqrsvtuyawxbcedf
ghijklmnoprqstuvwxaybcedfijklmnoprstuqvwxyhhabcdeggijkmfrfsabglhjmnoctdpqtewrxilkuunowsxpcdvefijgk
hmyyqabvlnopqsruvtwxyabcdefghijklnmopqrbwsctuavxyrelmowjkvxfnghisdtuqpaycbdefghijklmnopqrstuvwxyab
cdefhgijklmnoprqtsuvwxaybdcefghijklmnopqrstwuvxyacbdefgihjklmnopqrstuvwxyabcdfeghijklmnopqrstuvwxy
bacdfeghiklmjonqprstuvwxyabecdfghijklmnopqrstuvwcyabdxfeghikjlpnqmorstuvwyxabcdefghijklmnopqrstuvw
yxbadcfeighjlkmnopqrstuvwyxabcdefgijhklnmpoqrstuvwxyabcegdfihjklmnopqrstuvwxyabdcefghijklmpnoqstru
xyvwcabdmejghilknfopqryabdestuvwxfhcmqjrklignopycstuvwxabfdhkeomlgqrijpnsutvwxaybcdefghjiklnmopqrs
tuvwyxabcdefghijklmnopqsrtuvwxyabcdefghijklmnopqrstuvwxyabcdefghijkmlonpqrstvuxacywheibdfgjklmnopqrvsuwtxabycdefghijklmnoqprstvuwyxabcdefgihkjnlmpoqrstuvwyxabcdefghijklmnpoqrsutvwxyabcedfghjilnkmopqrestuvxfgwabcdhiykmjlqnoprstwyabucvxdeghfijklmnopqrstyuwxveabcdfghijklmnopqrstuvwxyabcdefghijlmopkqnsrtuvwxyabcedfghijklmnopqrstuvwxaybcdefghijklmnopqrstuvwxyabcdegfhijklnupwsybcxamoqrvdetmfghijklnpoqsrutwxyabcvdefghijlkmonpqrtsuvwxadcybefghijklnmopqrstuvwxyabcdefjghilknmpqorstwuxvydabcefghijknolmpqrsxtwvycafbuedghijklmnopqrutsvwxyabcdefghijklmopnqrtusvyxawbcdefghijklmnopqrstuvwyxbacedfghijklmnopqrstuvwxyacebdfghimnopjqklrstuvwxyabcfhdeijklgmnpqosurwxtvyabcdfeghjklmnoqpsrvtxubwyacdegfhjklmnopqrstuvwxaybcdfeghjklmnopqrsutvwxyaiiibcdefghijklmnopqbswuxvacdertyihfgjklmnopqrstuwvxybacdefghijklmnopqrstuvwxyabcdefhgijkmlnopqrstuvwxyabcdefghijklmnopqrstvuwxyabcdefghijklmnopqrstuvwxyacegbmdnhpfrijkloqsutwvbcdeaxfygihjklmnopqrstuvwxyabcdefghilkjmonqprstuvwyxabcdefhgijklmnopqrsutvwyxbacedfigkjhlmnpoqrstuvwxyabcdefghijklmoqputxrnvyaswbdfceghjiklmnoprqstuvwxabydecfghijkl
^C
```

[main.rs]: src/main.rs

## References

- [RabbitMQ]: The most widely deployed open source message broker
- [Crate lapin-futures]: [Crate futures v0.1] based [Crate lapin]
- [Crate lapin]: RabbitMQ crate based on AMQP 0.9.1 specification
- [The Async Book]: Asynchronous Programming in Rust
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
[the async book]: https://rust-lang.github.io/async-book/
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
