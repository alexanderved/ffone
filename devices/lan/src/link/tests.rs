#![allow(dead_code)]

use super::*;
use crate::network::*;

use core::audio_system::audio::{AudioCodec, MuxedAudioBuffer};
use core::util::RunnableStateMachine;
use std::net::{Ipv4Addr, TcpListener, TcpStream, UdpSocket};
use std::thread::{self, JoinHandle};

struct StopDevice;

impl Message for StopDevice {}

struct FakeDevice {
    name: String,
    audio_port: u16,

    listener: TcpListener,
    msg_stream: Option<TcpStream>,

    audio_listener_addr: Option<SocketAddr>,
    audio_stream: UdpSocket,
}

impl FakeDevice {
    const AUDIO_CODEC: AudioCodec = AudioCodec::Opus;
    const AUDIO_SAMPLE_RATE: u32 = 48000;

    fn new(
        name: &str,
        port: u16,
        audio_port: u16,
    ) -> error::Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))?;
        let audio_stream = UdpSocket::bind((Ipv4Addr::LOCALHOST, audio_port))?;
        audio_stream.set_nonblocking(true)?;

        Ok(Self {
            name: String::from(name),
            audio_port,

            listener,
            msg_stream: None,

            audio_listener_addr: None,
            audio_stream,
        })
    }

    fn info(&self) -> DeviceInfo {
        DeviceInfo {
            name: self.name.clone(),
        }
    }

    fn audio_port(&self) -> u16 {
        self.audio_port
    }
}

impl Runnable for FakeDevice {
    fn update(&mut self) -> error::Result<()> {
        if let Some(addr) = self.audio_listener_addr {
            let packet = NetworkPacket::from_bytes([42; 42].to_vec());
            self.audio_stream.send_packet_to(addr, &packet)?;
        }

        let mut msg_stream = self
            .msg_stream
            .as_ref()
            .expect("A message stream wasn't obtained");

        let packet = msg_stream.read_packet()?;
        let msg = packet.deserialize()?;

        let packet = match msg {
            HostMessage::Ping => NetworkPacket::serialize(&DeviceMessage::Pong),
            HostMessage::Connected { audio_port } => {
                let ip = msg_stream.peer_addr().unwrap().ip();
                self.audio_listener_addr = Some((ip, audio_port).into());
                return Ok(());
            }
        };

        msg_stream.write_packet(&packet?)?;

        Ok(())
    }

    fn on_start(&mut self) {
        let msg_stream = self.listener.accept().unwrap().0;
        msg_stream.set_nonblocking(true).unwrap();
        msg_stream.set_nodelay(true).unwrap();

        self.msg_stream = Some(msg_stream);
    }
}

fn run_device(
    name: &str,
    port: u16,
    audio_port: u16,
) -> error::Result<(MessageSender<StopDevice>, JoinHandle<()>)> {
    let (device_send, device_recv) = unidirectional_queue();
    let mut device = FakeDevice::new(name, port, audio_port)?;
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

fn create_link(
    msg_port: u16,
    audio_port: u16,
) -> error::Result<(
    RunnableStateMachine<LanLink>,
    MessageReceiver<DeviceSystemElementMessage>,
)> {
    let (link_send, link_recv) = unidirectional_queue();
    let mut link = LanLink::new(LanDeviceInfo::new(
        "fake",
        (Ipv4Addr::LOCALHOST, msg_port).into(),
        (Ipv4Addr::LOCALHOST, audio_port).into(),
    ))?;
    link.connect(link_send);
    let mut link = RunnableStateMachine::new(link);
    link.start()?;

    Ok((link, link_recv))
}

#[test]
fn test_on_info_received() -> error::Result<()> {
    let device_port = 31709;
    let audio_port = 31710;
    let (device_send, device_handle) = run_device("fake", device_port, audio_port)?;
    let (mut link, _link_recv) = create_link(device_port, audio_port)?;

    let mut info = DeviceInfo::new("");
    while let Some(_) = link.proceed() {
        info = link.runnable().info();
        break;
    }
    link.stop()?;

    assert_eq!(info, DeviceInfo::new("fake"));

    stop_device((device_send, device_handle));

    Ok(())
}

#[test]
fn test_on_encoded_audio_received() -> error::Result<()> {
    let device_port = 31711;
    let audio_port = 31712;
    let (device_send, device_handle) = run_device("fake", device_port, audio_port)?;
    let (mut link, link_recv) = create_link(device_port, audio_port)?;

    let mut muxed_audio_buffer = MuxedAudioBuffer(vec![]);
    while let Some(_) = link.proceed() {
        if let Some(DeviceSystemElementMessage::MuxedAudioReceived(buf)) = link_recv.recv() {
            muxed_audio_buffer = buf;

            break;
        }
    }
    link.stop()?;

    assert_eq!(muxed_audio_buffer, MuxedAudioBuffer(vec![42; 42]));

    stop_device((device_send, device_handle));

    Ok(())
}
