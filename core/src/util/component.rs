use crate::error;

use mueue::*;

pub trait Component {
    type Message: Message;
    type ControlMessage: Message;

    fn endpoint(&self) -> MessageEndpoint<Self::ControlMessage, Self::Message>;
    fn connect(&mut self, end: MessageEndpoint<Self::ControlMessage, Self::Message>);

    fn send(&self, msg: Self::Message) {
        let _ = self.endpoint().send(msg);
    }
}

pub trait ComponentBuilder {
    type Component: Component + ?Sized;

    fn set_endpoint(
        &mut self,
        end: MessageEndpoint<
            <Self::Component as Component>::ControlMessage,
            <Self::Component as Component>::Message,
        >,
    );
    fn build(self: Box<Self>) -> error::Result<Box<Self::Component>>;
}
