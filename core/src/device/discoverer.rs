use super::link::DeviceLink;
use super::DeviceInfo;

use crate::error;
use crate::util::*;

use mueue::*;

pub type DeviceDiscovererEndpoint =
    MessageEndpoint<DeviceDiscovererControlMessage, DeviceDiscovererMessage>;

pub enum DeviceDiscovererMessage {
    DevicesEnumerated(Box<dyn Iterator<Item = DeviceInfo> + Send + Sync>),

    LinkOpened(Box<dyn DeviceLink>),
    OpenLinkError(error::Error),

    NewDevicesDiscovered(Box<dyn Iterator<Item = DeviceInfo> + Send + Sync>),
    DeviceUnreachable(DeviceInfo),
    Error(error::Error),
}

impl Message for DeviceDiscovererMessage {}

pub enum DeviceDiscovererControlMessage {
    EnumerateDevices,
    OpenLink(DeviceInfo),

    Stop,
}

impl Message for DeviceDiscovererControlMessage {}

pub trait DeviceDiscoverer:
    Component<Message = DeviceDiscovererMessage, ControlMessage = DeviceDiscovererControlMessage>
    + Runnable
    + Send
{
    fn enumerate_devices(&self) -> Box<dyn Iterator<Item = DeviceInfo> + Send + Sync>;

    fn open_link(&mut self, info: DeviceInfo) -> error::Result<Box<dyn DeviceLink>>;
}

crate::impl_control_message_handler! {
    @component DeviceDiscoverer;
    @message DeviceDiscovererMessage;
    @control_message DeviceDiscovererControlMessage;

    EnumerateDevices => enumerate_devices => DevicesEnumerated;
    OpenLink(info) => open_link => @map_or_else(OpenLinkError, LinkOpened);
}
