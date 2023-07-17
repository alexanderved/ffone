use super::element::*;
use super::link::DeviceLink;
use super::DeviceInfo;

use crate::error;
use crate::util::*;

pub type DeviceDiscovererStateMachine = RunnableStateMachine<Box<dyn DeviceDiscoverer>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceDiscovererInfo {
    pub name: String,
}

pub trait DeviceDiscoverer: DeviceSystemElement + Send {
    fn info(&self) -> DeviceDiscovererInfo;
    fn enumerate_devices(&self) -> Box<dyn Iterator<Item = DeviceInfo> + Send + Sync>;
    fn open_link(&mut self, info: DeviceInfo) -> error::Result<Box<dyn DeviceLink>>;
}

crate::trait_alias!(pub DeviceDiscovererBuilder:
    DeviceSystemElementBuilder<Element = dyn DeviceDiscoverer>);
