use super::*;
use crate::network::*;

use core::util::RunnableStateMachine;
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

        let packet = link.read_packet()?;
        let msg = packet.deserialize()?;

        let packet = match msg {
            HostMessage::Ping => NetworkPacket::serialize(&DeviceMessage::Pong),
            HostMessage::GetAudioPort => NetworkPacket::serialize(&DeviceMessage::AudioPort {
                port: Self::AUDIO_PORT,
            }),
        };

        link.write_packet(&packet?)?;

        Ok(())
    }

    fn on_start(&mut self) -> error::Result<()> {
        let mut link = self.listener.accept()?.0;
        link.set_nonblocking(true)?;
        link.set_nodelay(true)?;

        link.write_packet(&NetworkPacket::serialize(&DeviceMessage::Info {
            info: self.info(),
        })?)?;
        link.write_packet(&NetworkPacket::serialize(&DeviceMessage::AudioPort {
            port: self.audio_port(),
        })?)?;

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

fn create_link(
    port: u16,
) -> error::Result<(
    RunnableStateMachine<LanLink>,
    MessageReceiver<DeviceSystemElementMessage>,
)> {
    let (link_send, link_recv) = unidirectional_queue();
    let mut link = LanLink::new(LanDeviceInfo::new(
        "fake",
        (Ipv4Addr::LOCALHOST, port).into(),
    ))?;
    link.connect(link_send);
    let link = RunnableStateMachine::new_running(link).map_err(|(_, err)| err)?;

    Ok((link, link_recv))
}

#[test]
fn test_on_info_received() -> error::Result<()> {
    let device_port = 31707;
    let (device_send, device_handle) = run_device("fake", device_port)?;
    let (mut link, _link_recv) = create_link(device_port)?;

    let mut info = DeviceInfo::new("");
    while let Some(_) = link.proceed() {
        info = link.as_runnable().info();
        break;
    }
    link.stop()?;

    assert_eq!(info, DeviceInfo::new("fake"));

    stop_device((device_send, device_handle));

    Ok(())
}

#[test]
fn test_on_audio_port_received() -> error::Result<()> {
    let device_port = 31708;
    let (device_send, device_handle) = run_device("fake", device_port)?;
    let (mut link, _link_recv) = create_link(device_port)?;

    let mut addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0));
    while let Some(_) = link.proceed() {
        if let Some(audio_stream) = link.as_runnable().audio_stream.as_ref() {
            let Ok(address) = audio_stream.socket().peer_addr() else {
                continue;
            };
            addr = address;
            break;
        }
    }
    link.stop()?;

    assert_eq!(
        addr,
        SocketAddr::from((Ipv4Addr::LOCALHOST, FakeDevice::AUDIO_PORT))
    );

    stop_device((device_send, device_handle));

    Ok(())
}