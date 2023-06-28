use std::net::SocketAddr;

use mueue::{Message, MessageEndpoint};

type DeviceLinkEndpoint = MessageEndpoint<DeviceControlMessage, DeviceMessage>;

pub enum DeviceMessage {}

impl Message for DeviceMessage {}

pub enum DeviceControlMessage {}

impl Message for DeviceControlMessage {}

pub trait DeviceLink {
    fn is_linked(&self) -> bool;
    fn link(&mut self, addr: SocketAddr);

    fn endpoint(&self) -> DeviceLinkEndpoint;
    fn connect(&mut self, end: DeviceLinkEndpoint);

    fn update(&mut self);

    fn run(&mut self) {
        loop {
            self.update();
        }
    }
}
