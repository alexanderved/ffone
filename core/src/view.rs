use mueue::{Message, MessageEndpoint};

pub enum ViewRequest {
    
}

impl Message for ViewRequest {}

pub trait View {
    fn connect_controller(&mut self, end: MessageEndpoint);
    fn send_message(&self, msg: ViewRequest);

    fn update(&mut self);

    fn run(&mut self) {
        loop {
            self.update();
        }
    }
}