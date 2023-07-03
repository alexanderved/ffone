use super::*;

use core::device::link::*;
use core::device::*;
use core::error;
use core::util::{Component, ControlFlow, Runnable};

use core::mueue::*;

use std::io::{Read, Write};
use std::net::TcpStream;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DeviceMessage {
    pub msg: String,
}

impl DeviceMessage {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: String::from(msg),
        }
    }
}

pub struct LanLink {
    end: Option<DeviceLinkEndpoint>,
    info: LanDeviceInfo,
    link: TcpStream,
}

impl LanLink {
    pub fn new(info: LanDeviceInfo) -> error::Result<Self> {
        Ok(Self {
            end: None,
            info: info.clone(),
            link: TcpStream::connect(info.addr)?,
        })
    }

    pub fn audio_transmission_port(&mut self) -> error::Result<u16> {
        let get = DeviceMessage::new("get_audio_transmission_port");

        let mut bytes = serde_json::to_vec(&get)?;
        self.link.write(&bytes)?;

        let bytes_len = self.link.read(&mut bytes)?;
        let port = serde_json::from_slice(&bytes[..bytes_len])?;

        Ok(port)
    }
}

impl Component for LanLink {
    type Message = DeviceLinkMessage;
    type ControlMessage = DeviceLinkControlMessage;

    fn endpoint(&self) -> MessageEndpoint<Self::ControlMessage, Self::Message> {
        self.end.clone().expect("A device link endpoint wasn't set")
    }

    fn connect(&mut self, end: MessageEndpoint<Self::ControlMessage, Self::Message>) {
        self.end = Some(end);
    }
}

impl Runnable for LanLink {
    fn update(&mut self, _flow: &mut ControlFlow) {
        self.endpoint().iter().for_each(|_msg| todo!());

        todo!()
    }
}

impl DeviceLink for LanLink {
    fn info(&self) -> DeviceInfo {
        self.info.info()
    }
}
