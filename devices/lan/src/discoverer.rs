#[cfg(test)]
mod tests;

use super::broadcast::*;
use super::*;

use crate::link::LanLink;

use core::device::discoverer::*;
use core::device::element::DeviceSystemElementMessage;
use core::device::link::*;
use core::device::*;
use core::error;
use core::mueue::*;
use core::util::ControlFlow;
use core::util::Element;
use core::util::Runnable;

use std::collections::HashMap;
use std::collections::HashSet;

pub const BROADCAST_PORT: u16 = 31703;

pub struct LanDiscoverer {
    send: MessageSender<DeviceSystemElementMessage>,
    infos: HashMap<DeviceInfo, LanDeviceInfo>,

    broadcast: UdpBroadcastListener,
}

impl LanDiscoverer {
    pub fn new(send: MessageSender<DeviceSystemElementMessage>) -> error::Result<Self> {
        Ok(Self {
            send,
            infos: HashMap::new(),

            broadcast: UdpBroadcastListener::new(BROADCAST_PORT)?,
        })
    }

    fn discover_devices(
        &mut self,
    ) -> error::Result<Box<dyn Iterator<Item = DeviceInfo> + Send + Sync>> {
        let mut new_devices = HashSet::new();
        self.broadcast.recv()?.for_each(|lan_info| {
            if self
                .infos
                .insert(lan_info.info(), lan_info.clone())
                .is_some()
            {
                new_devices.insert(lan_info.info());
            }
        });

        Ok(Box::new(new_devices.into_iter()))
    }
}

impl Element for LanDiscoverer {
    type Message = DeviceSystemElementMessage;

    fn sender(&self) -> core::mueue::MessageSender<Self::Message> {
        self.send.clone()
    }

    fn connect(&mut self, send: MessageSender<DeviceSystemElementMessage>) {
        self.send = send;
    }
}

impl Runnable for LanDiscoverer {
    fn update(&mut self, _flow: Option<&mut ControlFlow>) -> error::Result<()> {
        let msg = self.discover_devices().map_or_else(
            DeviceSystemElementMessage::Error,
            DeviceSystemElementMessage::NewDevicesDiscovered,
        );
        self.send(msg);

        Ok(())
    }
}

impl DeviceDiscoverer for LanDiscoverer {
    fn info(&self) -> DeviceDiscovererInfo {
        DeviceDiscovererInfo {
            name: "Lan Device Discoverer".to_string(),
        }
    }

    fn enumerate_devices(&self) -> Box<dyn Iterator<Item = DeviceInfo> + Send + Sync> {
        Box::new(self.infos.clone().into_keys())
    }

    fn open_link(&mut self, info: DeviceInfo) -> error::Result<Box<dyn DeviceLink>> {
        let lan_info = self.infos.get(&info).ok_or(error::Error::NoDevice)?.clone();
        let link = LanLink::new(lan_info);

        if link.is_err() {
            self.infos.remove(&info);
            self.send(DeviceSystemElementMessage::DeviceUnreachable(info));
        }

        Ok(Box::new(link?))
    }
}
