use crate::network::*;

use super::*;

use core::device::link::*;
use core::device::*;
use core::error;
use core::util::{Component, ControlFlow, Runnable};

use core::mueue::*;

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
        let packet = NetworkPacket::serialize(&cmd)?;
        packet.send(&self.link)?;

        let packet = NetworkPacket::recv(&self.link)?;
        Ok(packet.deserialize()?)
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
        if is_tcp_socket_disconnected(&self.link) {
            self.send(DeviceLinkMessage::DeviceDisconnected);
            *flow = ControlFlow::Break;

            //return Ok(());
        }
        //panic!("");

        self.endpoint()
            .iter()
            .for_each(|msg| {
                dbg!(&msg);
                msg.handle(self, &mut *flow);
            });

        Ok(())
    }
}

impl DeviceLink for LanLink {
    fn info(&self) -> DeviceInfo {
        self.info.info()
    }

    fn audio_address(&self) -> error::Result<SocketAddr> {
        let ip = self.link.peer_addr()?.ip();
        let port = self.get_from_device::<u16>(DeviceCommand::GetAudioTransmissionPort)?;

        Ok(SocketAddr::from((ip, port)))
    }
}

fn is_tcp_socket_disconnected(socket: &TcpStream) -> bool {
    use std::io;

    let res: Result<_, io::Error> = core::try_block! {
        let mut buf = [0; 16];
        socket.set_nonblocking(true)?;
        let n = socket.peek(&mut buf)?;
        socket.set_nonblocking(false)?;

        Ok(n)
    };

    //panic!("{:?}", &res);
    dbg!(&res);
    
    match res {
        Ok(0) => true,
        Err(err) if err.kind() == io::ErrorKind::ConnectionAborted => true,
        Err(err) if err.kind() == io::ErrorKind::ConnectionReset => true,
        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => true,
        Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

            let link = self.link.as_ref().expect("A link wasn't obtained");

            let packet = NetworkPacket::recv(&link)?;
            let msg = packet.deserialize()?;

            let packet = match msg {
                DeviceCommand::GetInfo => NetworkPacket::serialize(&self.info())?,
                DeviceCommand::GetAudioTransmissionPort => {
                    NetworkPacket::serialize(&self.audio_port())?
                }
                _ => return Ok(()),
            };
            packet.send(&link)?;

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

    //#[test]
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

    //#[test]
    fn test_get_audio_transmission_address() -> error::Result<()> {
        let device_port = 31708;
        let (device_send, device_handle) = run_device("fake", device_port)?;
        let (link_end, link_handle) = run_link(device_port)?;

        let _ = link_end.send(DeviceLinkControlMessage::GetAudioAddress);
        /* let addr = 'outer: loop {
            for msg in link_end.iter() {
                if let DeviceLinkMessage::AudioAddress(addr) = msg {
                    break 'outer addr;
                };
            }
        };
        assert_eq!(
            addr,
            SocketAddr::from((Ipv4Addr::LOCALHOST, FakeDevice::AUDIO_PORT))
        ); */

        stop_device((device_send, device_handle));
        stop_link((link_end, link_handle));

        Ok(())
    }
}
