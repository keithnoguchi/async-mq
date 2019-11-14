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
        struct Test {
            name: &'static str,
            count: usize,
        }
        let tests = [
            Test {
                name: "single oneshot",
                count: 1,
            },
            Test {
                name: "100 oneshots",
                count: 100,
            },
            Test {
                name: "1,000 oneshots",
                count: 1_000,
            },
        ];
        for t in &tests {
            use futures::future::lazy;
            let name = t.name;
            let count = t.count;
            tokio::run(lazy(move || {
                use futures::{future::Future, sync::oneshot};
                for _ in 0..count {
                    let (tx, rx) = oneshot::channel();
                    tokio::spawn(lazy(move || {
                        tx.send(format!("{}", name)).unwrap();
                        Ok(())
                    }));
                    tokio::spawn(lazy(move || {
                        rx.and_then(|msg| {
                            println!("[{}] got it!", msg);
                            Ok(())
                        })
                        .map_err(move |err| eprintln!("[{}] error: {}", name, err))
                    }));
                }
                Ok(())
            }))
        }
    }
}
