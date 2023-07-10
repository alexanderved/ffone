use super::link::DeviceLink;

use crate::error;
use crate::util::*;

use mueue::*;

pub enum DeviceLinkStorageMessage {
    Device(Box<dyn DeviceLink>),
    NoDevice(error::Error),
}

impl Message for DeviceLinkStorageMessage {}

pub enum DeviceLinkStorageControlMessage {
    Store(Box<dyn DeviceLink>),
    Load,

    Stop,
}

impl Message for DeviceLinkStorageControlMessage {}

pub struct DeviceLinkStorage {
    end: Option<MessageEndpoint<DeviceLinkStorageControlMessage, DeviceLinkStorageMessage>>,
    link: Option<Box<dyn DeviceLink>>,
    link_control_flow: ControlFlow,
}

impl DeviceLinkStorage {
    pub fn store(&mut self, mut link: Box<dyn DeviceLink>) {
        self.link_control_flow = ControlFlow::Continue;
        let _ = link.on_start();

        self.link = Some(link);
    }

    pub fn load(&mut self) -> error::Result<Box<dyn DeviceLink>> {
        self.link
            .take()
            .map(|mut link| {
                let _ = link.on_stop();
                link
            })
            .ok_or(error::Error::NoDevice)
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
    fn update(&mut self, flow: &mut ControlFlow) -> error::Result<()> {
        self.link.as_mut().map(|link| {
            if matches!(self.link_control_flow, ControlFlow::Continue) {
                let _ = link.update(&mut self.link_control_flow);
            }
        });

        self.endpoint()
            .iter()
            .for_each(|msg| msg.handle(self, &mut *flow));

        Ok(())
    }
}

crate::impl_control_message_handler! {
    @concrete_component DeviceLinkStorage;
    @message DeviceLinkStorageMessage;
    @control_message DeviceLinkStorageControlMessage;

    Store(link) => store;
    Load => load => @ok Device, @err NoDevice;
}
