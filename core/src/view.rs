use crate::controller::ViewControlMessage;

use std::sync::Arc;

use mueue::{Message, MessageEndpoint};

type ControllerEndpoint = MessageEndpoint<ViewControlMessage, ViewRequest>;

pub enum ViewRequest {}

impl Message for ViewRequest {}

pub trait View {
    fn controller_endpoint(&self) -> ControllerEndpoint;
    fn connect(&mut self, end: ControllerEndpoint);

    fn update(&mut self);

    fn send(&self, msg: ViewRequest) {
        let _ = self.controller_endpoint().send(Arc::new(msg));
    }

    fn run(&mut self) {
        loop {
            self.update();
        }
    }
}
