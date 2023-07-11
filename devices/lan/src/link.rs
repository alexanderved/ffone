use crate::connection::TcpConnection;

use super::*;

use core::device::link::*;
use core::device::*;
use core::error;
use core::util::{Component, ControlFlow, Runnable, Timer};

use core::mueue::*;

use std::time::Duration;

pub struct LanLink {
    end: Option<DeviceLinkEndpoint>,
    info: LanDeviceInfo,

    link: TcpConnection,
    check_connection_timer: Timer,
}

impl LanLink {
    pub fn new(info: LanDeviceInfo) -> error::Result<Self> {
        let link = TcpConnection::new(info.addr)?;

        Ok(Self {
            end: None,
            info: info.clone(),

            link,
            check_connection_timer: Timer::new(Duration::from_secs(1)),
        })
    }

    pub fn get_from_device<U>(&mut self, cmd: DeviceCommand) -> error::Result<U>
    where
        U: for<'de> serde::Deserialize<'de> + std::fmt::Debug,
    {
        self.link.send(&cmd)?;
        let res = self.link.recv()?;

        Ok(res)
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
    fn update(&mut self, flow: &mut ControlFlow) -> error::Result<()> {
        self.check_connection_timer.on_timeout(|| {
            if !self.link.is_open() {
                *flow = ControlFlow::Break;
                self.send(DeviceLinkMessage::DeviceDisconnected);
            }
        });

        self.endpoint()
            .iter()
            .for_each(|msg| msg.handle(self, &mut *flow));

        Ok(())
    }
}

impl DeviceLink for LanLink {
    fn info(&self) -> DeviceInfo {
        self.info.info()
    }

    fn audio_address(&mut self) -> error::Result<SocketAddr> {
        let ip = self.link.socket().peer_addr()?.ip();
        let port = self.get_from_device::<u16>(DeviceCommand::GetAudioPort)?;

        Ok(SocketAddr::from((ip, port)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::*;

    use std::net::{Ipv4Addr, TcpListener, TcpStream};
    use std::thread::{self, JoinHandle};

    struct StopDevice;

    impl Message for StopDevice {}

    struct FakeDevice {
        recv: MessageReceiver<StopDevice>,
        name: String,

        listener: TcpListener,
        link: Option<TcpStream>,
    }

    impl FakeDevice {
        const AUDIO_PORT: u16 = 31709;

        fn new(name: &str, recv: MessageReceiver<StopDevice>, port: u16) -> error::Result<Self> {
            let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))?;

            Ok(Self {
                recv,
                name: String::from(name),

                listener,
                link: None,
            })
        }

        fn info(&self) -> DeviceInfo {
            DeviceInfo {
                name: self.name.clone(),
            }
        }

        fn audio_port(&self) -> u16 {
            Self::AUDIO_PORT
        }
    }

    impl Runnable for FakeDevice {
        fn update(&mut self, flow: &mut ControlFlow) -> error::Result<()> {
            if matches!(self.recv.recv(), Some(StopDevice)) {
                *flow = ControlFlow::Break;
                return Ok(());
            }

            let mut link = self.link.as_ref().expect("A link wasn't obtained");

            let packet = link.recv_packet()?;
            let msg = packet.deserialize()?;

            let packet = match msg {
                DeviceCommand::GetInfo => NetworkPacket::serialize(&self.info())?,
                DeviceCommand::GetAudioPort => {
                    NetworkPacket::serialize(&self.audio_port())?
                }
                _ => return Ok(()),
            };

            link.send_packet(&packet)?;

            Ok(())
        }

        fn on_start(&mut self) -> error::Result<()> {
            let link = self.listener.accept()?.0;
            link.set_nonblocking(true)?;
            link.set_nodelay(true)?;

            self.link = Some(link);

            Ok(())
        }
    }

    fn run_device(
        name: &str,
        port: u16,
    ) -> error::Result<(MessageSender<StopDevice>, JoinHandle<()>)> {
        let (device_send, device_recv) = unidirectional_queue();
        let mut device = FakeDevice::new(name, device_recv, port)?;
        let device_handle = thread::spawn(move || {
            let _ = device.run();
        });

        Ok((device_send, device_handle))
    }

    fn stop_device((device_send, device_handle): (MessageSender<StopDevice>, JoinHandle<()>)) {
        let _ = device_send.send(StopDevice);
        device_handle.join().unwrap();
    }

    fn run_link(
        port: u16,
    ) -> error::Result<(
        MessageEndpoint<DeviceLinkMessage, DeviceLinkControlMessage>,
        JoinHandle<()>,
    )> {
        let mut link = LanLink::new(LanDeviceInfo::new(
            "fake",
            (Ipv4Addr::LOCALHOST, port).into(),
        ))?;
        let (link_end, link_end1) = bidirectional_queue();
        link.connect(link_end1);

        let link_handle = thread::spawn(move || {
            let _ = link.run();
        });

        Ok((link_end, link_handle))
    }

    fn stop_link(
        (link_end, link_handle): (
            MessageEndpoint<DeviceLinkMessage, DeviceLinkControlMessage>,
            JoinHandle<()>,
        ),
    ) {
        let _ = link_end.send(DeviceLinkControlMessage::Stop);
        link_handle.join().unwrap();
    }

    #[test]
    fn test_get_info() -> error::Result<()> {
        let device_port = 31707;
        let (device_send, device_handle) = run_device("fake", device_port)?;
        let (link_end, link_handle) = run_link(device_port)?;

        let _ = link_end.send(DeviceLinkControlMessage::GetInfo);
        let info = 'outer: loop {
            for msg in link_end.iter() {
                if let DeviceLinkMessage::Info(info) = msg {
                    break 'outer info;
                };
            }
        };
        assert_eq!(info, DeviceInfo::new("fake"));

        stop_device((device_send, device_handle));
        stop_link((link_end, link_handle));

        Ok(())
    }

    #[test]
    fn test_get_audio_address() -> error::Result<()> {
        let device_port = 31708;
        let (device_send, device_handle) = run_device("fake", device_port)?;
        let (link_end, link_handle) = run_link(device_port)?;

        let _ = link_end.send(DeviceLinkControlMessage::GetAudioAddress);

        let addr = 'outer: loop {
            for msg in link_end.iter() {
                if let DeviceLinkMessage::AudioAddress(addr) = msg {
                    break 'outer addr;
                };
            }
        };
        assert_eq!(
            addr,
            SocketAddr::from((Ipv4Addr::LOCALHOST, FakeDevice::AUDIO_PORT))
        );

        stop_device((device_send, device_handle));
        stop_link((link_end, link_handle));

        Ok(())
    }
}
