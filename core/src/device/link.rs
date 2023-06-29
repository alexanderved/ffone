use crate::util::{Component, Runnable};

use mueue::Message;

pub enum DeviceLinkMessage {}

impl Message for DeviceLinkMessage {}

pub enum DeviceLinkControlMessage {}

impl Message for DeviceLinkControlMessage {}

pub trait DeviceLink:
    Component<Message = DeviceLinkMessage, ControlMessage = DeviceLinkControlMessage> + Runnable
{
}
