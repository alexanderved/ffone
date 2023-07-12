pub mod discoverer;
pub mod element;
pub mod link;
pub mod storage;

use discoverer::*;
use link::*;
use mueue::{Message, MessageEndpoint};

pub type DeviceSystemEndpoint = MessageEndpoint<DeviceSystemControlMessage, DeviceSystemMessage>;

pub enum DeviceSystemMessage {}

impl Message for DeviceSystemMessage {}

pub enum DeviceSystemControlMessage {}

impl Message for DeviceSystemControlMessage {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DeviceInfo {
    pub name: String,
}

impl DeviceInfo {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
#[serde(tag = "type")]
pub enum HostMessage {
    Ping,
    Pong,

    GetInfo,
    GetAudioPort,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
#[serde(tag = "type")]
pub enum DeviceMessage {
    Ping,
    Pong,

    Info { info: DeviceInfo },
    AudioPort { port: u16 },
}

pub struct DeviceSystem {
    end: DeviceSystemEndpoint,

    active_discoverer: Option<DeviceDiscovererStateMachine>,
    discoverers: Vec<Box<dyn DeviceDiscoverer>>,

    link: Option<DeviceLinkStateMachine>,
}

/* impl DeviceSystem {
    pub fn new(end: DeviceSystemEndpoint, discoverers)
} */
