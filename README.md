# rustmq

Zero-cost [lapin] abstraction crate

[lapin]: https://crates.io/crates/lapin

[![DroneCI]](https://cloud.drone.io/keithnoguchi/rustmq)
[![CircleCI]](https://circleci.com/gh/keithnoguchi/workflows/rustmq)

[DroneCI]: https://cloud.drone.io/api/badges/keithnoguchi/rustmq/status.svg
[CircleCI]: https://circleci.com/gh/keithnoguchi/rustmq.svg?style=svg

## Modules

- [client]: `Client` and `Connection` structs
- [consume]: `Consumer`, `ConsumerBuilder`, and `ConsumerExt` trait
- [produce]: `Producer`, `ProducerBuilder`, and `ProducerExt` trait

[client]: src/client.rs
[consume]: src/consume.rs
[produce]: src/produce.rs
[flatbuffers]: https://google.github.io/flatbuffers/

## Example

Currently, [main.rs] demonstrates the RabbitMQ RPC pattern
through the Rust 1.39 [async-await] feature with [lapin].
It creates 32 producer threads and 8 consumer threads, with each
thread runs 8 consumer [async-await] instances.  It also uses
[FlatBuffers] for the message encoding.

[main.rs]: src/main.rs
[async-await]: https://blog.rust-lang.org/2019/11/07/Async-await-stable.html

Here is the main function which creates threads both the producers
and consumers:

```sh
fn main() -> thread::Result<()> {
    let client = Client::new();
    let queue_name = "hello";
    let uri = parse();

    // A single connection for the multiple producers.
    let conn = block_on(client.connect(&uri)).expect("fail to connect");
    let mut builder = ProducerBuilder::new(conn);
    builder.queue(String::from(queue_name));
    let mut producers = Vec::with_capacity(PRODUCER_THREAD_NR);
    for _ in 0..producers.capacity() {
        let builder = builder.clone();
        let producer = thread::spawn(move || {
            ASCIIGenerator::new(builder).run().expect("generator died");
        });
        producers.push(producer);
    }

    // A single connection for multiple consumers.
    let conn = block_on(client.connect(&uri)).expect("fail to connect");
    let mut builder = ConsumerBuilder::new(conn);
    builder.queue(String::from(queue_name));
    let mut consumers = Vec::with_capacity(CONSUMER_THREAD_NR);
    for _ in 0..consumers.capacity() {
        let builder = builder.clone();
        let consumer = thread::spawn(move || {
            LocalConsumerManager::new(builder, CONSUMER_INSTANCE_NR).run();
        });
        consumers.push(consumer);
    }

    // Cleanup all instances.
    while !producers.is_empty() {
        let producer = producers.pop().unwrap();
        producer.join()?;
    }
    while !consumers.is_empty() {
        let consumer = consumers.pop().unwrap();
        consumer.join()?;
    }
    Ok(())
}
```

Here are the producer side of the structures:

```sh
struct ASCIIGenerator {
    builder: ProducerBuilder,
}

impl ASCIIGenerator {
    fn new(builder: rustmq::ProducerBuilder) -> Self {
        Self { builder }
    }
    fn run(&mut self) -> Result<()> {
        let builder = self.builder.clone();
        let mut pool = LocalPool::new();
        pool.run_until(async move {
            let mut buf_builder = FlatBufferBuilder::new();
            let mut p = builder.build().await?;
            p.with_ext(Box::new(FlatBufferPrinter {}));
            loop {
                for data in { b'!'..b'~' } {
                    let data = buf_builder.create_string(&String::from_utf8(vec![data]).unwrap());
                    let mut mb = crate::msg::MessageBuilder::new(&mut buf_builder);
                    mb.add_msg(data);
                    let msg = mb.finish();
                    buf_builder.finish(msg, None);
                    let msg = buf_builder.finished_data();
                    p.rpc(msg.to_vec()).await?;
                    buf_builder.reset();
                }
            }
        })
    }
}

#[derive(Clone)]
pub struct FlatBufferPrinter;

#[async_trait]
impl rustmq::ProducerExt for FlatBufferPrinter {
    async fn recv(&mut self, msg: Vec<u8>) -> lapin::Result<()> {
        let msg = crate::msg::get_root_as_message(&msg);
        if let Some(msg) = msg.msg() {
            eprint!("{}", msg);
        }
        Ok(())
    }
    fn box_clone(&self) -> Box<dyn rustmq::ProducerExt + Send> {
        Box::new((*self).clone())
    }
}
```

And are the consumer side:

```sh
struct LocalConsumerManager {
    builder: ConsumerBuilder,
    consumers: usize,
    pool: LocalPool,
    spawner: LocalSpawner,
}

impl LocalConsumerManager {
    fn new(builder: rustmq::ConsumerBuilder, consumers: usize) -> Self {
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
        let mut builder = self.builder.clone();
        let consumers = self.consumers;
        let spawner = self.spawner.clone();
        self.spawner
            .spawn_local(async move {
                builder.with_ext(Box::new(EchoMessage {}));
                for _ in 0..consumers {
                    let mut consumer = builder.build().await.expect("consumer build failed");
                    let _task = spawner.spawn_local(async move {
                        consumer.run().await.expect("consumer died");
                    });
                }
            })
            .expect("consumer manager died");
        self.pool.run();
    }
}

#[derive(Clone)]
struct EchoMessage;

#[async_trait]
impl rustmq::ConsumerExt for EchoMessage {
    async fn recv(&mut self, msg: Vec<u8>) -> lapin::Result<Vec<u8>> {
        Ok(msg)
    }
    fn box_clone(&self) -> Box<dyn rustmq::ConsumerExt + Send> {
        Box::new((*self).clone())
    }
}
```

## Execution

Here is the output of the cargo run.  It dumps printable ascii
characters to the stderr.

```sh
$ cargo run
   Compiling rustmq v0.3.0 (/home/kei/git/rustmq)
    Finished dev [unoptimized + debuginfo] target(s) in 1.63s
     Running `target/debug/rustmq`
!!!!!!!!!!!"!!"!!!!!!!!!"!!""!!!!!!!"!""""#"""#""""#""#"""""#""#"#"##""#"$##"$#######$######$$###$##$$$$$%$$%#$#$$$$$$%$$$
$$$%%$$$%$%%%$%%&%&%%$%%%%%%%&%%%%&%&%&%&&%&'&%&%%&&&&&&&&''&'&&&&&'&&'''(&&&'''('''''('&'''''(''(&'(''('()'()('(()((()(((
()'()()(()((()*'((*()*())))*)**))))))))(+)*))*))(+*)*+*)*+*+*****+*,****+)**+*+**+,)*+,*,+++,+,++++-,+++*++++,+*-,,++,-,,-
,+,,-,,-,,-,,+,-,,,-,..+--.,---.------,.,,-..---,.--/.-//..-......-.//.--./...-//...0.///0////././/00////100.//0/.00//0.01
/010/00020110/0000100/101121100/100121112111120131112012223122221122231223022323343131322332223423534244332442344313234333
3343465444535445543345434444555425475556665455466555666447554568365766556766755676766579787876767458875677666667768788789:
978788987577898677887688798899:99;989:8:869988899:99998:98:77;:::::;7:9;9<99;9;::9:;;;9;:<::;8:8:9=:;;<;8::<<::;<:;:=<9:<9
9<<;<=;;><<;<;;;;=;=:<;=;>=;<;==:<<?=<=<><>><<<=<=:=;=><><<@=<==>?>;?===>>;?<?===>===?>?>>A=>?<?=?=?@>>><@>>@@?B=?>>?@>>>@
?@>?>>@>@??=@AA?CAA??@?@A?AA?@?@@A??@??A>?B@>AB@D@B@@BA@BABBA@B@C@AAEAA@CAB@C?@A@A?BAACBCCAACBBADCBDABBBAEB@BAFBDCDBCDDBBC
BDCADB@CCEBCCBCCFGEEAACDDCBCFEECCBEDDCDDDCDECCDFHGDBFGDDFEFBEFEDIHDDEDDCEGEGDEEECEFCDHEEGFCFIHFFDEEDEEFEFFHJEEFFDFGFGIFHGD
GGJIGEFFEHGFIHEHJHFGGGFKGIHFGFGJGKGEHFGGFGIJIKHIJIHHHFIGHHGKGHIHGFHHLGLHGHJKJIJJILGIHJHIKGLJMJHIKHIIKILHIHIKIIMKJMHJKNJHKJ
LLJKMIKJIIJJKILNNIMLLJJJIILLKJMIKKKJMMOKKLMKMJNLJKMNJOJOLJMLKNKJLLNKLNKLONPNOKPKLMLKPNLMMMNMKLOKLMOMLLOOMPLMPLLNMONNNMQLMP
MNPLMNQOOQPNMPNOMNMMNPQQOOOMMPRNPQROONNNQONOQPRONNQONPPRNPRQNSQPORQPOPOSORSQQSQPORPQOOSOPORORRTQPSRSTPSQQURTPQPPRTRPPTQRPQ
SRPSTQSSPUTSRQVRTSSUTQQQRQTQTTUUUTRSQQRSSVURQWRUSSTRTVUVXUVTRRRURSRTSTVVRRVWUUSSTTUSVWTSSVWSUTVSWVTXUTVVUSWSWWUTUSYWUXUTWT
UWVXWTUTVWXYTVXUXXUWUXTVVYVTVZXUVWXXUYWVUXYZYZUWVXYYYW[UWWWVVUYYVXWYVZVZWZX[YV[XXWX\VYZWXWZ[YVWXXWZZWZZ[ZW[Y\YXY\X[XXZ\X]W
XZY[[WZ\Y]X[\[Y[[YXZ]YYY\Z][Y^X[YZY]Z\X]\Z[ZZ^\_[\YY^Z\Z[\]^\Z[YZZ\[^]Y\][^[]Z[Z_]`\_\[[Z]]_\][[Z\^_]^][\^\[[^\`_a^^[]^\\]
^^_]\[\^]\`\`]__]`\_]ba`_\_^`_]_`c_^a\ba]]a^`^`a]_]^]^]`^]`a``^_dc^__^bab`_^a`]ab^^aaa^___^bab_`ed__``cbab_`^cb_`b`e_ab_cb
bfc`a`_c``aadcb`cd_c`a`cdcagacdeafaabecdba`c``bddabdabedhbabdedadedbabcfbcbacgeeecbffbceiebdchdbeegeccfgfcffcccfbdgdfbfdjc
cdefefcichhdeddedegfddggfdgckddghefggjggdieeeifledeehehgehhghehehfhgfiffkfhjfjhiifimiggiikffgfffiigjfhkiiegjhglgjgjnhjgggj
jgkjmkjijlkhhkhlghghihjhofkihhhkhkkkmilniglijlpjlliikhilhikiimjiillmlhmommjmijjjjqknjnljmmjkjjimnjkmlinnnnpmkknnkrkjklomkn
ookojkoknkoklljoqnllllskloplpokollpkpplmpolmnplrmommmplmlmmqplqqmqmpnstomqpnqmmqnnpnnqmnmnqnmrnnonrrotonrnrurqqoronpoqonro
onrsoooppsoussspptpsrppvooppsoprporssqqpqtpvtqsttqttptpwqptqqpqqsrurqqqsruqwuuurqrvurrttuuvsurrrsvqqxrsvtxrrqrvswrsvssuvuv
sswwwrtsyvsxswvrrttwtytsussttvwwtxtxuuswuzwxsuuvxywutxtx
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
[async in traits]: https://rust-lang.github.io/async-book/07_workarounds/06_async_in_traits.html
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
