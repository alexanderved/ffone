use super::broadcast::*;
use super::*;

use crate::link::LanLink;

use core::device::discoverer::*;
use core::device::link::*;
use core::device::*;
use core::error;
use core::util::ControlFlow;
use core::util::{Component, Runnable};

use std::collections::HashMap;
use std::collections::HashSet;

pub const BROADCAST_PORT: u16 = 31703;

pub struct LanDiscoverer {
    end: Option<DeviceDiscovererEndpoint>,
    infos: HashMap<DeviceInfo, LanDeviceInfo>,

    broadcast: UdpBroadcastListener,
}

impl LanDiscoverer {
    pub fn new() -> error::Result<Self> {
        Ok(Self {
            end: None,
            infos: HashMap::new(),

            broadcast: UdpBroadcastListener::new(BROADCAST_PORT)?,
        })
    }

    fn discover_devices(
        &mut self,
    ) -> error::Result<Box<dyn Iterator<Item = DeviceInfo> + Send + Sync>> {
        let mut new_devices = HashSet::new();
        self.broadcast.recv()?.for_each(|lan_info| {
            self.infos
                .insert(lan_info.info(), lan_info.clone())
                .map(|_| {
                    new_devices.insert(lan_info.info());
                });
        });

        Ok(Box::new(new_devices.into_iter()))
    }
}

impl Component for LanDiscoverer {
    type Message = DeviceDiscovererMessage;
    type ControlMessage = DeviceDiscovererControlMessage;

    fn endpoint(&self) -> DeviceDiscovererEndpoint {
        self.end
            .clone()
            .expect("A device discoverer endpoint wasn't set")
    }

    fn connect(&mut self, end: DeviceDiscovererEndpoint) {
        self.end = Some(end);
    }
}

impl Runnable for LanDiscoverer {
    fn update(&mut self, flow: &mut ControlFlow) -> error::Result<()> {
        let msg = self.discover_devices().map_or_else(
            DeviceDiscovererMessage::Error,
            DeviceDiscovererMessage::NewDevicesDiscovered,
        );
        self.send(msg);

        self.endpoint()
            .iter()
            .for_each(|msg| msg.handle(self, &mut *flow));

        Ok(())
    }
}

impl DeviceDiscoverer for LanDiscoverer {
    fn enumerate_devices(&self) -> Box<dyn Iterator<Item = DeviceInfo> + Send + Sync> {
        Box::new(self.infos.clone().into_iter().map(|(k, _)| k))
    }

    fn open_link(&mut self, info: DeviceInfo) -> error::Result<Box<dyn DeviceLink>> {
        let lan_info = self.infos.get(&info).ok_or(error::Error::NoDevice)?.clone();
        let link = LanLink::new(lan_info);

        if link.is_err() {
            self.infos.remove(&info);
            self.send(DeviceDiscovererMessage::DeviceUnreachable(info));
        }

        Ok(Box::new(link?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::*;

    use core::mueue::*;

    use std::collections::HashSet;
    use std::net::UdpSocket;
    use std::net::{Ipv4Addr, SocketAddr};
    use std::thread::{self, JoinHandle};

    struct StopDevice;

    impl Message for StopDevice {}

    struct FakeDevice {
        recv: MessageReceiver<StopDevice>,
        name: String,
        broadcast_socket: UdpSocket,
    }

    impl FakeDevice {
        const PORT: u16 = 31707;

        fn new(name: &str, recv: MessageReceiver<StopDevice>) -> error::Result<Self> {
            let broadcast_socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))?;
            broadcast_socket.set_broadcast(true)?;

            Ok(Self {
                recv,
                name: String::from(name),
                broadcast_socket,
            })
        }
    }

    impl Runnable for FakeDevice {
        fn update(&mut self, flow: &mut ControlFlow) -> error::Result<()> {
            if matches!(self.recv.recv(), Some(StopDevice)) {
                *flow = ControlFlow::Break;
                return Ok(());
            }

            let identity_packet = IdentityPacket {
                name: self.name.clone(),
                port: Self::PORT,
            };
            let data = NetworkPacket::serialize(&identity_packet)?;

            let _ = self.broadcast_socket.send_packet_to(
                SocketAddr::from((Ipv4Addr::BROADCAST, BROADCAST_PORT)),
                &data,
            );

            Ok(())
        }
    }

    fn run_device(name: &str) -> error::Result<(MessageSender<StopDevice>, JoinHandle<()>)> {
        let (device_send, device_recv) = unidirectional_queue();
        let mut device = FakeDevice::new(name, device_recv)?;
        let device_handle = thread::spawn(move || {
            let _ = device.run();
        });

        Ok((device_send, device_handle))
    }

    fn stop_device((device_send, device_handle): (MessageSender<StopDevice>, JoinHandle<()>)) {
        let _ = device_send.send(StopDevice);
        device_handle.join().unwrap();
    }

    fn run_disoverer() -> error::Result<(
        MessageEndpoint<DeviceDiscovererMessage, DeviceDiscovererControlMessage>,
        JoinHandle<()>,
    )> {
        let mut discoverer = LanDiscoverer::new()?;
        let (disc_end, disc_end1) = bidirectional_queue();
        discoverer.connect(disc_end1);

        let disc_handle = thread::spawn(move || {
            let _ = discoverer.run();
        });

        Ok((disc_end, disc_handle))
    }

    fn stop_discoverer(
        (disc_send, disc_handle): (
            MessageEndpoint<DeviceDiscovererMessage, DeviceDiscovererControlMessage>,
            JoinHandle<()>,
        ),
    ) {
        let _ = disc_send.send(DeviceDiscovererControlMessage::Stop);
        disc_handle.join().unwrap();
    }

    #[test]
    fn test_enumerate_devices() -> error::Result<()> {
        let (device_send, device_handle) = run_device("fake")?;
        let (disc_end, disc_handle) = run_disoverer()?;
        let (device_send1, device_handle1) = run_device("fake1")?;

        let _ = disc_end.send(DeviceDiscovererControlMessage::EnumerateDevices);

        let mut infos = HashSet::new();
        while infos.len() < 2 {
            for msg in disc_end.iter() {
                match msg {
                    DeviceDiscovererMessage::DevicesEnumerated(devs) => {
                        infos.clear();
                        infos.extend(devs);
                    }
                    DeviceDiscovererMessage::NewDevicesDiscovered(devs) => {
                        infos.extend(devs);
                    }
                    _ => unimplemented!(),
                }
            }
        }

        assert!(infos.contains(&DeviceInfo::new("fake")), "{:?}", infos);
        assert!(infos.contains(&DeviceInfo::new("fake1")), "{:?}", infos);

        stop_device((device_send, device_handle));
        stop_device((device_send1, device_handle1));
        stop_discoverer((disc_end, disc_handle));

        Ok(())
    }
}
