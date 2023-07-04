mod broadcast;
pub mod discoverer;
pub mod link;

use core::device::DeviceInfo;

use std::net::SocketAddr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LanDeviceInfo {
    pub info: DeviceInfo,
    pub addr: SocketAddr,
}

impl LanDeviceInfo {
    pub fn new(name: &str, addr: SocketAddr) -> Self {
        Self {
            info: DeviceInfo::new(name),
            addr,
        }
    }

    pub fn info(&self) -> DeviceInfo {
        self.info.clone()
    }
}
