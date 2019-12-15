# rustmq

Zero-cost abstraction [lapin] abstraction [AMQP] client crate

[lapin]: https://crates.io/crates/lapin
[amqp]: https://www.amqp.org

[![DroneCI]](https://cloud.drone.io/keithnoguchi/rustmq)
[![CircleCI]](https://circleci.com/gh/keithnoguchi/workflows/rustmq)

[DroneCI]: https://cloud.drone.io/api/badges/keithnoguchi/rustmq/status.svg
[CircleCI]: https://circleci.com/gh/keithnoguchi/rustmq.svg?style=svg

## Modules

- [client]: `Client` and `Connection` structs
- [consume]: `Consumer` and `ConsumerBuilder` structs
- [produce]: `Producer`, `ProducerBuilder`, and `ProducerHandler` trait

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
    let mut threads = Vec::with_capacity(PRODUCER_THREAD_NR + CONSUMER_THREAD_NR);
    let client = Client::new();
    let queue_name = "request";
    let uri = parse();

    // A single connection for the multiple producers.
    let conn = block_on(client.connect(&uri)).expect("fail to connect");
    let mut builder = conn.producer_builder();
    builder.queue(String::from(queue_name));
    for _ in 0..PRODUCER_THREAD_NR {
        let builder = builder.clone();
        let producer = thread::spawn(move || {
            ASCIIGenerator::new(builder).run().expect("generator died");
        });
        threads.push(producer);
    }

    // A single connection for multiple consumers.
    let conn = block_on(client.connect(&uri)).expect("fail to connect");
    let mut builder = conn.consumer_builder();
    builder.queue(String::from(queue_name));
    for _ in 0..CONSUMER_THREAD_NR {
        let builder = builder.clone();
        let consumer = thread::spawn(move || {
            LocalEchoConsumer::new(builder, CONSUMER_INSTANCE_NR).run();
        });
        threads.push(consumer);
    }

    // Cleanup all instances.
    for t in threads {
        t.join()?;
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
    fn new(builder: ProducerBuilder) -> Self {
        Self { builder }
    }
    fn run(&mut self) -> Result<(), Error> {
        let builder = self.builder.clone();
        let mut pool = LocalPool::new();
        pool.run_until(async move {
            let mut p = builder.build().await?;
            let mut builder = FlatBufferBuilder::new();
            loop {
                // Generate ASCII character FlatBuffer messages
                // and print the received message to stderr.
                for data in { b'!'..=b'~' } {
                    let req = self.make_buf(&mut builder, vec![data]);
                    let resp = p.rpc(req).await?;
                    self.print_buf(resp);
                }
            }
        })
    }
    fn make_buf(&self, builder: &mut FlatBufferBuilder, data: Vec<u8>) -> Vec<u8> {
        let data = builder.create_string(&String::from_utf8(data).unwrap());
        let mut mb = crate::msg::MessageBuilder::new(builder);
        mb.add_msg(data);
        let msg = mb.finish();
        builder.finish(msg, None);
        let req = builder.finished_data().to_vec();
        builder.reset();
        req
    }
    fn print_buf(&self, resp: Vec<u8>) {
        if resp.is_empty() {
            return;
        }
        let msg = crate::msg::get_root_as_message(&resp);
        if let Some(data) = msg.msg() {
            eprint!("{}", data);
        }
    }
}
```

And the consumer side:

```sh
struct LocalEchoConsumer {
    builder: ConsumerBuilder,
    consumers: usize,
    pool: LocalPool,
    spawner: LocalSpawner,
}

impl LocalEchoConsumer {
    fn new(builder: ConsumerBuilder, consumers: usize) -> Self {
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
        let builder = self.builder.clone();
        let spawner = self.spawner.clone();
        let consumers = self.consumers;
        self.spawner
            .spawn_local(async move {
                for _ in 0..consumers {
                    let mut c = builder.build().await.expect("consumer build failed");
                    let _task = spawner.spawn_local(async move {
                        while let Some(Ok(req)) = c.next().await {
                            // Echo back the message.
                            if let Err(err) = c.response(&req, req.data()).await {
                                eprintln!("{}", err);
                            }
                        }
                    });
                }
            })
            .expect("consumer manager died");
        self.pool.run();
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

You can check the queue status with `rabbitmqctl list_queues` as below:

```sh
$  sudo rabbitmqctl list_queues --vhost your_vhost
Timeout: 60.0 seconds ...
Listing queues for vhost mx ...
name    messages
amq.gen-3CNgzxmjJGoTIjAcy2zhHQ  0
amq.gen-6kHFOKiqnJIltqeH-1I4WQ  0
amq.gen-kWjXOMz0MDX9Rwo6F8sPCA  0
amq.gen-tFNZuCMpdn6so9WnNLAS4w  1
amq.gen-ScKfpco30LHj1feWIIsFXg  1
amq.gen-7uGgXCiiExxrAXZEcrbypg  0
amq.gen-vkFA83xnTHHhR6c_zUG34A  0
amq.gen-48HpeKkKhmQLAa4Q1beQvg  0
amq.gen-FAYzlhiy9liKuMEUVUm2Uw  0
amq.gen-de6sYdZX8cT8yYkb_Y-mPw  0
hello   23
amq.gen-gu_TDgPgWcestqlWNtFSoA  0
amq.gen-advkCsBJKod22vexRSBV6A  0
amq.gen-T6jLPqqOKsL9jk04CFJwtA  0
amq.gen--wRWW5hI-rpdds9goMAYdg  1
amq.gen-f9EYMuzwQoCAhNyIg6SiUQ  0
amq.gen-DxmcpUxHYGOxCD9Q7QM1xg  1
amq.gen-EeeJAmHFOT3GPIMhfXDi8Q  0
amq.gen-zhqOTQag0rM7MDU17pCdXA  0
amq.gen-vyX6_lAv2Pnmcm1a_tBUKA  0
amq.gen-enXP4BCXZhB0tg4C1an4sw  0
amq.gen-nIpKSKokUzU_pCoGTiSBCQ  0
amq.gen-k1p4udDFoA0xhqSXIpPo-Q  0
amq.gen-XrNQZ0cqHgSgUZK_CP0g6w  0
amq.gen-9WZJ4Jw02Dbhhl7sJIQKAQ  0
amq.gen-1TMw_E8g09Xt9UgCoMz-ig  0
amq.gen-rVUkIh-85ims0IiabvF7GA  1
amq.gen-9NeGc4C9qmfX-PLjHkXVDA  0
amq.gen-swJrMKZmNnLhtI0Djz--ag  0
amq.gen-5rEAFqYpG4cp8lBEDG6_gQ  1
amq.gen-ZDNy2Ggt4Dqbvj6cnS-c8A  0
amq.gen-D8id7SF143eN-k7tmHratw  1
amq.gen-AiKQaNiz73V9du8EtgKfMg  0
```

## References

- [The Async Book]: The Asynchronous Programming in Rust
  - [Async in traits]
  - [Crate async-trait]
  - [Why async fn in traits are hard]
  - [Async Destructors]
- [The Style Book]: The Rust Style Guidelines
- [RabbitMQ]: The most widely deployed open source message broker
- [Crate lapin]: RabbitMQ crate based on AMQP 0.9.1 specification
  - [Lapin futures 0.3 example]: [Crate futures 0.3] example.
- [Crate futures 0.3]: Abstructions for Asynchronous Programming
  - [Rust streams] by [Yoshua Wuyts]
- [crate tokio 0.2]
  - [A Tour of Tokio]
  - [The Tokio Book]
  - [Making the Tokio 0.2 scheduler 10x faster]
  - [crate Metal I/O]
  - [tokio 0.1 internals]
- [Original futures design]: Original futures design by [Aaron Turon]

Happy Hacking!

[the async book]: https://rust-lang.github.io/async-book/
[async in traits]: https://rust-lang.github.io/async-book/07_workarounds/06_async_in_traits.html
[crate async-trait]: https://github.com/dtolnay/async-trait
[why async fn in traits are hard]: https://smallcultfollowing.com/babysteps/blog/2019/10/26/async-fn-in-traits-are-hard/
[async Destructors]: https://boats.gitlab.io/blog/post/poll-drop/
[the style book]: https://doc.rust-lang.org/1.0.0/style/README.html
[RabbitMQ]: https://www.rabbitmq.com
[crate lapin]: https://docs.rs/lapin/0.28.2/lapin/
[lapin futures 0.3 example]: https://github.com/sozu-proxy/lapin/blob/master/examples/pubsub_futures.rs
[crate futures 0.3]: https://docs.rs/futures/0.3.1/
[rust streams]: https://blog.yoshuawuyts.com/rust-streams/
[Yoshua Wuyts]: https://blog.yoshuawuyts.com/
[crate tokio 0.2]: https://tokio.rs/blog/2019-11-tokio-0-2/
[a tour of tokio]: https://docs.rs/tokio/latest/tokio/#a-tour-of-tokio
[the tokio book]: https://github.com/tokio-rs/book/blob/master/SUMMARY.md
[making the tokio 0.2 scheduler 10x faster]: https://tokio.rs/blog/2019-10-scheduler/
[Crate Metal I/O]: https://github.com/tokio-rs/mio
[tokio 0.1 internals]: https://cafbit.com/post/tokio_internals/
[original futures design]: https://aturon.github.io/blog/2016/09/07/futures-design/
[Aaron Turon]: https://aturon.github.io/blog/
