use super::DeviceInfo;
use super::link::DeviceLink;

use crate::util::{Component, Runnable};

use mueue::*;

pub enum DeviceDiscovererMessage {}

impl Message for DeviceDiscovererMessage {}

pub enum DeviceDiscovererControlMessage {
    EnumerateDevices,
    OpenLink(DeviceInfo),
    CloseLink(DeviceInfo),
}

impl Message for DeviceDiscovererControlMessage {}

pub trait DeviceDiscoverer:
    Component<Message = DeviceDiscovererMessage, ControlMessage = DeviceDiscovererControlMessage>
    + Runnable
{
    fn enumerate_devices(&self) -> Box<dyn Iterator<Item = DeviceInfo> + '_>;

    fn open_link(&self, info: DeviceInfo) -> Box<dyn DeviceLink>;
    fn close_link(&self, link: Box<dyn DeviceLink>);
}
