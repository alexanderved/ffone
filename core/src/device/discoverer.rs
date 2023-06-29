use crate::util::{Component, Runnable};

use mueue::*;

pub enum DeviceDiscovererMessage {}

impl Message for DeviceDiscovererMessage {}

pub enum DeviceDiscovererControlMessage {}

impl Message for DeviceDiscovererControlMessage {}

pub trait DeviceDiscoverer:
    Component<Message = DeviceDiscovererMessage, ControlMessage = DeviceDiscovererControlMessage>
    + Runnable
{
}
