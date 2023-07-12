use crate::error;

use mueue::*;

pub trait Element {
    type Message: Message;

    fn sender(&self) -> MessageSender<Self::Message>;
    fn connect(&mut self, send: MessageSender<Self::Message>);

    fn send(&self, msg: Self::Message) {
        let _ = self.sender().send(msg);
    }
}

pub trait ElementBuilder {
    type Element: Element + ?Sized;

    fn set_sender(&mut self, send: MessageSender<<Self::Element as Element>::Message>);
    fn build(self: Box<Self>) -> error::Result<Box<Self::Element>>;
}
