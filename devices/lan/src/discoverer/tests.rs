use super::*;
use crate::network::*;
use core::util::RunnableStateMachine;

use std::collections::HashSet;
use std::net::*;
use std::thread::{self, JoinHandle};

struct StopDevice;

impl Message for StopDevice {}

struct FakeDevice {
    name: String,
    broadcast_socket: UdpSocket,
}

impl FakeDevice {
    const PORT: u16 = 31707;
    const AUDIO_PORT: u16 = 31708;

    fn new(name: &str) -> error::Result<Self> {
        let broadcast_socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))?;
        broadcast_socket.set_broadcast(true)?;

        Ok(Self {
            name: String::from(name),
            broadcast_socket,
        })
    }
}

impl Runnable for FakeDevice {
    fn update(&mut self) -> error::Result<()> {
        let identity_packet = IdentityPacket {
            name: self.name.clone(),
            msg_port: Self::PORT,
            audio_port: Self::AUDIO_PORT,
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
    let mut device = FakeDevice::new(name)?;
    let device_handle = thread::spawn(move || {
        device.on_start();
        while device_recv.recv().is_none() {
            let _ = device.update();
        }
        device.on_stop();
    });

    Ok((device_send, device_handle))
}

fn stop_device((device_send, device_handle): (MessageSender<StopDevice>, JoinHandle<()>)) {
    let _ = device_send.send(StopDevice);
    device_handle.join().unwrap();
}

#[test]
fn test_enumerate_devices() -> error::Result<()> {
    let (device_send, device_handle) = run_device("fake")?;
    let (device_send1, device_handle1) = run_device("fake1")?;

    let (disc_send, _disc_recv) = unidirectional_queue();
    let mut discoverer = RunnableStateMachine::new(LanDiscoverer::new(disc_send)?);
    discoverer.start()?;

    let mut infos = HashSet::new();
    while infos.len() < 2 {
        if let Some(_) = discoverer.proceed() {
            infos.extend(discoverer.runnable().enumerate_devices())
        }
    }
    discoverer.stop()?;

    assert!(infos.contains(&DeviceInfo::new("fake")), "{:?}", infos);
    assert!(infos.contains(&DeviceInfo::new("fake1")), "{:?}", infos);

    stop_device((device_send, device_handle));
    stop_device((device_send1, device_handle1));

    Ok(())
}
