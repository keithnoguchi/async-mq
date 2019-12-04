// SPDX-License-Identifier: GPL-2.0
#![allow(
    unused_imports,
    clippy::extra_unused_lifetimes,
    clippy::needless_lifetimes,
    clippy::redundant_closure,
    clippy::redundant_static_lifetimes
)]
include!("../schema/model_generated.rs");

pub use model::get_root_as_message;
pub use model::{Message, MessageArgs, MessageBuilder};

#[cfg(test)]
mod tests {
    use flatbuffers::FlatBufferBuilder;
    #[test]
    fn message_create() {
        use super::get_root_as_message;
        use super::{Message, MessageArgs};
        let names = ["a", "b", "c", "d"];
        for name in &names {
            let mut b = FlatBufferBuilder::new();
            let msg_name = b.create_string(name);
            let msg = Message::create(
                &mut b,
                &MessageArgs {
                    name: Some(msg_name),
                },
            );
            b.finish(msg, None);
            let buf = b.finished_data();
            let got = get_root_as_message(buf);
            match got.name() {
                Some(got) => assert_eq!(name, &got),
                _ => panic!("unexpected None"),
            }
        }
    }
}
