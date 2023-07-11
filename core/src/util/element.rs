use crate::error;

use mueue::*;

pub trait Element {
    type Notiication: Message;

    fn sender(&self) -> MessageSender<Self::Notiication>;
    fn connect(&mut self, send: MessageSender<Self::Notiication>);

    fn send(&self, msg: Self::Notiication) {
        let _ = self.sender().send(msg);
    }
}

pub trait ElementBuilder {
    type Element: Element + ?Sized;

    fn set_sender(&mut self, send: MessageSender<<Self::Element as Element>::Notiication>);
    fn build(self: Box<Self>) -> error::Result<Box<Self::Element>>;
}
