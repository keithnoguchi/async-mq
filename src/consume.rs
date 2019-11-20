// SPDX-License-Identifier: GPL-2.0
use lapin::message::DeliveryResult;
use lapin::options::BasicAckOptions;
use lapin::{Channel, ConsumerDelegate};

#[derive(Clone, Debug)]
pub struct Consumer {
    chan: Channel,
}

impl Consumer {
    pub fn new(chan: Channel) -> Self {
        Self { chan }
    }
}

impl ConsumerDelegate for Consumer {
    fn on_new_delivery(&self, delivery: DeliveryResult) {
        if let Some(delivery) = delivery.unwrap() {
            print!(".");
            self.chan
                .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                .wait()
                .expect("basic_ack")
        }
    }
}
