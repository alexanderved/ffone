use super::link::DeviceLink;

use crate::util::{Component, ControlFlow, Runnable};

use mueue::*;

pub enum DeviceLinkStorageMessage {
    Device(Box<dyn DeviceLink>),
}

impl Message for DeviceLinkStorageMessage {}

pub enum DeviceLinkStorageControlMessage {
    Store(Box<dyn DeviceLink>),
    Load,
}

impl Message for DeviceLinkStorageControlMessage {}

pub struct DeviceLinkStorage {
    end: Option<MessageEndpoint<DeviceLinkStorageControlMessage, DeviceLinkStorageMessage>>,
    link: Option<Box<dyn DeviceLink>>,
    link_control_flow: ControlFlow,
}

impl DeviceLinkStorage {
    pub fn store(&mut self, link: Box<dyn DeviceLink>) {
        self.link = Some(link);
    }

    pub fn load(&mut self) -> Option<Box<dyn DeviceLink>> {
        self.link.take()
    }
}

impl Component for DeviceLinkStorage {
    type Message = DeviceLinkStorageMessage;
    type ControlMessage = DeviceLinkStorageControlMessage;

    fn endpoint(&self) -> MessageEndpoint<Self::ControlMessage, Self::Message> {
        self.end.clone().expect("A message endpoint wasn't set")
    }

    fn connect(&mut self, end: MessageEndpoint<Self::ControlMessage, Self::Message>) {
        self.end = Some(end);
    }
}

impl Runnable for DeviceLinkStorage {
    fn update(&mut self, _flow: &mut ControlFlow) {
        self.link.as_mut().map(|link| {
            if matches!(self.link_control_flow, ControlFlow::Continue) {
                link.update(&mut self.link_control_flow);
            }
        });

        self.endpoint()
            .iter()
            .for_each(|msg| match msg {
                DeviceLinkStorageControlMessage::Store(link) => self.store(link),
                DeviceLinkStorageControlMessage::Load => {
                    self.load().map(|link| {
                        self.send(DeviceLinkStorageMessage::Device(link));
                    });
                }
            });
    }
}
