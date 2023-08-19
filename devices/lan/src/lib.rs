mod audio_stream;
mod broadcast;
pub mod discoverer;
pub mod link;
mod message_stream;
mod network;
mod poller;

use core::{audio_system::audio::EncodedAudioInfo, device::DeviceInfo};

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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
#[serde(tag = "type")]
pub enum HostMessage {
    Ping,
    Empty,

    AudioListenerStarted { port: u16 },
    RestartAudioStream,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
#[serde(tag = "type")]
pub enum DeviceMessage {
    Pong,

    Info { info: DeviceInfo },
    StartAudioListener { port: u16, info: EncodedAudioInfo },
    StopAudioListener,
}
