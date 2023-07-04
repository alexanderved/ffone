use super::*;

use core::device::link::*;
use core::device::*;
use core::error;
use core::util::{Component, ControlFlow, Runnable};

use core::mueue::*;

use std::io::Read;
use std::io::Write;
use std::net::TcpStream;

pub struct LanLink {
    end: Option<DeviceLinkEndpoint>,
    info: LanDeviceInfo,

    link: TcpStream,
}

impl LanLink {
    pub fn new(info: LanDeviceInfo) -> error::Result<Self> {
        let link = TcpStream::connect(info.addr)?;
        link.set_nodelay(true)?;

        Ok(Self {
            end: None,
            info: info.clone(),

            link,
        })
    }

    pub fn get_from_device<U>(&self, cmd: DeviceCommand) -> error::Result<U>
    where
        U: for<'de> serde::Deserialize<'de>,
    {
        let mut bytes = serde_json::to_vec(&cmd)?;
        (&self.link).write(&bytes)?;

        bytes.reserve(128);
        let len = (&self.link).read(&mut bytes)?;

        Ok(serde_json::from_slice(&bytes[0..len]).unwrap())
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
        self.endpoint().iter().for_each(|msg| match msg {
            DeviceLinkControlMessage::GetInfo => {
                self.send(DeviceLinkMessage::Info(self.info()));
            }
            DeviceLinkControlMessage::GetAudioTransmissionAddress => {
                let Ok(addr) = self.audio_transmission_address() else {
                    return;
                };
                self.send(DeviceLinkMessage::AudioTransmissionAddress(addr));
            }
            DeviceLinkControlMessage::Stop => {
                *flow = ControlFlow::Break;
            }
            _ => {},
        });

        Ok(())
    }
}

impl DeviceLink for LanLink {
    fn info(&self) -> DeviceInfo {
        self.info.info()
    }

    fn audio_transmission_address(&self) -> error::Result<SocketAddr> {
        let ip = self.link.peer_addr()?.ip();
        let port = self.get_from_device::<u16>(DeviceCommand::GetAudioTransmissionPort)?;

        Ok(SocketAddr::from((ip, port)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::{Read, Write};
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

            let mut buf = [0u8; 128];
            let len = link.read(&mut buf)?;

            let msg = serde_json::from_slice::<DeviceCommand>(&buf[..len])?;
            let buf = match msg {
                DeviceCommand::GetInfo => serde_json::to_vec(&self.info())?,
                DeviceCommand::GetAudioTransmissionPort => serde_json::to_vec(&self.audio_port())?,
                _ => return Ok(()),
            };
            link.write(&buf)?;

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

    fn run_link(port: u16) -> error::Result<(
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
    fn test_get_audio_transmission_address() -> error::Result<()> {
        let device_port = 31708;
        let (device_send, device_handle) = run_device("fake", device_port)?;
        let (link_end, link_handle) = run_link(device_port)?;

        let _ = link_end.send(DeviceLinkControlMessage::GetAudioTransmissionAddress);
        let addr = 'outer: loop {
            for msg in link_end.iter() {
                if let DeviceLinkMessage::AudioTransmissionAddress(addr) = msg {
                    break 'outer addr;
                };
            }
        };
        assert_eq!(addr, SocketAddr::from((Ipv4Addr::LOCALHOST, FakeDevice::AUDIO_PORT)));

        stop_device((device_send, device_handle));
        stop_link((link_end, link_handle));

        Ok(())
    }
}
