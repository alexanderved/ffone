pub extern crate mueue;

pub mod audio_system;
pub mod controller;
pub mod device_link;
pub mod view;

use std::sync::Arc;

use mueue::{Message, MessageEndpoint};

pub trait Component {
    type Message: Message;
    type ControlMessage: Message;

    fn endpoint(&self) -> MessageEndpoint<Self::ControlMessage, Self::Message>;
    fn connect(&mut self, end: MessageEndpoint<Self::ControlMessage, Self::Message>);

    fn send(&self, msg: Self::Message) {
        let _ = self.endpoint().send(Arc::new(msg));
    }
}

pub trait Runnable {
    fn update(&mut self);

    fn run(&mut self) {
        loop {
            self.update();
        }
    }
}
