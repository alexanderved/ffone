use super::DeviceInfo;

use crate::util::{Component, Runnable};

use mueue::{Message, MessageEndpoint};

pub type DeviceLinkEndpoint = MessageEndpoint<DeviceLinkControlMessage, DeviceLinkMessage>;

pub enum DeviceLinkMessage {
    Info(DeviceInfo),
}

impl Message for DeviceLinkMessage {}

pub enum DeviceLinkControlMessage {
    GetInfo,
}

impl Message for DeviceLinkControlMessage {}

pub trait DeviceLink:
    Component<Message = DeviceLinkMessage, ControlMessage = DeviceLinkControlMessage>
    + Runnable
    + Send
    + Sync
{
    fn info(&self) -> DeviceInfo;
}
