# rustmq

[RabbitMQ] with [Crate lapin] for fun.

[![DroneCI]](https://cloud.drone.io/keithnoguchi/rustmq)
[![CircleCI]](https://circleci.com/gh/keithnoguchi/workflows/rustmq)

[DroneCI]: https://cloud.drone.io/api/badges/keithnoguchi/rustmq/status.svg
[CircleCI]: https://circleci.com/gh/keithnoguchi/rustmq.svg?style=svg

## Modules

- [client]: For struct Client
- [consume]: For struct Consumer
- [publish]: For Struct Publisher
- [monster]: [Flatbuffers] tutorial example

[client]: src/client.rs
[consume]: src/consume.rs
[publish]: src/publish.rs
[monster]: src/monster.rs
[main.rs]: src/main.rs
[flatbuffers]: https://google.github.io/flatbuffers/

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
