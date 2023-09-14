mod audio_stream;
mod broadcast;
pub mod discoverer;
pub mod link;
mod message_stream;
mod network;
mod poller;

use core::device::DeviceInfo;

use std::net::SocketAddr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LanDeviceInfo {
    pub info: DeviceInfo,
    pub msg_addr: SocketAddr,
    pub audio_addr: SocketAddr,
}

impl LanDeviceInfo {
    pub fn new(name: &str, msg_addr: SocketAddr, audio_addr: SocketAddr) -> Self {
        Self {
            info: DeviceInfo::new(name),
            msg_addr,
            audio_addr,
        }
    }

    pub fn info(&self) -> DeviceInfo {
        self.info.clone()
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
#[serde(tag = "type")]
pub enum HostMessage {
    Ping,

    Connected { audio_port: u16 },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
#[serde(tag = "type")]
pub enum DeviceMessage {
    Pong,

    Info { info: DeviceInfo },
}
