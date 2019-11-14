// SPDX-License-Identifier: GPL-2.0
#[cfg(test)]
mod tests {
    use futures;
    #[test]
    fn lazy() {
        struct Test {
            name: &'static str,
            count: usize,
        }
        let tests = [
            Test {
                name: "four lazy tasks",
                count: 4,
            },
            Test {
                name: "100 lazy tasks",
                count: 100,
            },
        ];
        for t in &tests {
            let name = t.name;
            let count = t.count;
            tokio::run(futures::future::lazy(move || {
                for i in 0..count {
                    tokio::spawn(futures::future::lazy(move || {
                        println!("[{}]: task #{}", name, i);
                        Ok(())
                    }));
                }
                Ok(())
            }));
        }
    }
    #[test]
    fn oneshot() {
        use futures::future::{lazy, Future};
        tokio::run(lazy(|| {
            let (tx, rx) = futures::sync::oneshot::channel();
            tokio::spawn(lazy(|| {
                tx.send("sending it").unwrap();
                Ok(())
            }));
            rx.and_then(|msg| {
                println!("Got {}!", msg);
                Ok(())
            })
            .map_err(|err| eprintln!("receive error: {}", err))
        }));
    }
}
