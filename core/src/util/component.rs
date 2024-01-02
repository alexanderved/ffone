use crate::{error, util::RunnableStateMachine};

use std::thread::{self, JoinHandle};

use mueue::*;

use super::Runnable;

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

struct ComponentThreadStopMessage;

impl Message for ComponentThreadStopMessage {}

pub struct ComponentThread {
    handle: Option<JoinHandle<()>>,
    send: MessageSender<ComponentThreadStopMessage>,
}

impl ComponentThread {
    pub fn new<B>(builder: B) -> Self
    where
        B: ComponentBuilder + Send + 'static,
        B::Component: Runnable
    {
        let (send, recv) = unidirectional_queue();
        let handle = thread::spawn(move || {
            let Ok(component) = Box::new(builder).build() else {
                return;
            };
            let mut runnable_sm = RunnableStateMachine::new_running(component);

            while recv.recv().is_none() {
                let _ = runnable_sm.proceed();
            }
        });

        Self {
            handle: Some(handle),
            send,
        }
    }

    pub fn finish(mut self) {
        self.inner_finish();
    }

    fn inner_finish(&mut self) {
        let _ = self.send.send(ComponentThreadStopMessage);
        let _ = self.handle.take().unwrap().join();
    }
}

impl Drop for ComponentThread {
    fn drop(&mut self) {
        self.inner_finish();
    }
}