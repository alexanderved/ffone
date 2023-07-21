#![allow(dead_code)]

use super::*;
use crate::network::*;

use core::audio_system::EncodedAudioBuffer;
use core::util::RunnableStateMachine;
use std::net::{Ipv4Addr, TcpListener, TcpStream, UdpSocket};
use std::thread::{self, JoinHandle};

struct StopDevice;

impl Message for StopDevice {}

struct FakeDevice {
    recv: MessageReceiver<StopDevice>,
    name: String,
    audio_port: u16,

    listener: TcpListener,
    msg_stream: Option<TcpStream>,

    audio_listener_addr: Option<SocketAddr>,
    audio_stream: UdpSocket,
}

impl FakeDevice {
    const AUDIO_FORMAT: AudioFormat = AudioFormat::Rtp;
    const AUDIO_CODEC: AudioCodec = AudioCodec::Opus;

    fn new(
        name: &str,
        recv: MessageReceiver<StopDevice>,
        port: u16,
        audio_port: u16,
    ) -> error::Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))?;
        let audio_stream = UdpSocket::bind((Ipv4Addr::LOCALHOST, audio_port))?;
        audio_stream.set_nonblocking(true)?;

        Ok(Self {
            recv,
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
    fn update(&mut self, flow: &mut ControlFlow) -> error::Result<()> {
        if matches!(self.recv.recv(), Some(StopDevice)) {
            *flow = ControlFlow::Break;
            return Ok(());
        }

        if let Some(addr) = self.audio_listener_addr {
            self.audio_stream.send_to(&[42; 42], addr)?;
        }

        let mut msg_stream = self
            .msg_stream
            .as_ref()
            .expect("A message stream wasn't obtained");

        let packet = msg_stream.read_packet()?;
        let msg = packet.deserialize()?;

        let packet = match msg {
            HostMessage::Ping => NetworkPacket::serialize(&DeviceMessage::Pong),
            HostMessage::AudioListenerStarted { port } => {
                let ip = msg_stream.peer_addr().unwrap().ip();
                self.audio_listener_addr = Some((ip, port).into());
                return Ok(());
            }
            _ => return Ok(()),
        };

        msg_stream.write_packet(&packet?)?;

        Ok(())
    }

    fn on_start(&mut self) -> error::Result<()> {
        let mut msg_stream = self.listener.accept()?.0;
        msg_stream.set_nonblocking(true)?;
        msg_stream.set_nodelay(true)?;

        msg_stream.write_packet(&NetworkPacket::serialize(&DeviceMessage::AudioInfo {
            port: self.audio_port(),
            format: Self::AUDIO_FORMAT,
            codec: Self::AUDIO_CODEC,
        })?)?;

        self.msg_stream = Some(msg_stream);

        Ok(())
    }
}

fn run_device(
    name: &str,
    port: u16,
    audio_port: u16,
) -> error::Result<(MessageSender<StopDevice>, JoinHandle<()>)> {
    let (device_send, device_recv) = unidirectional_queue();
    let mut device = FakeDevice::new(name, device_recv, port, audio_port)?;
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
    let (device_send, device_handle) = run_device("fake", device_port, 31708)?;
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
fn test_on_audio_info_received() -> error::Result<()> {
    let device_port = 31709;
    let (device_send, device_handle) = run_device("fake", device_port, 31710)?;
    let (mut link, link_recv) = create_link(device_port)?;

    let mut format = AudioFormat::Unspecified;
    let mut codec = AudioCodec::Unspecified;
    let mut port = 0;
    while let Some(_) = link.proceed() {
        if let Some(DeviceSystemElementMessage::AudioInfoReceived(f, c)) = link_recv.recv() {
            format = f;
            codec = c;
            if let Some(audio_stream) = link.as_runnable().audio_stream.as_ref() {
                port = audio_stream.socket().peer_addr().map_or(0, |a| a.port());
            }

            break;
        }
    }
    link.stop()?;

    assert_eq!(format, FakeDevice::AUDIO_FORMAT);
    assert_eq!(codec, FakeDevice::AUDIO_CODEC);
    assert_eq!(port, 31710);

    stop_device((device_send, device_handle));

    Ok(())
}

#[test]
fn test_on_encoded_audio_received() -> error::Result<()> {
    let device_port = 31711;
    let (device_send, device_handle) = run_device("fake", device_port, 31712)?;
    let (mut link, link_recv) = create_link(device_port)?;

    let mut encoded_audio_buffer = EncodedAudioBuffer(vec![]);
    while let Some(_) = link.proceed() {
        if let Some(DeviceSystemElementMessage::EncodedAudioReceived(a)) = link_recv.recv() {
            encoded_audio_buffer = a;

            break;
        }
    }
    link.stop()?;

    assert_eq!(encoded_audio_buffer, EncodedAudioBuffer(vec![42; 42]));

    stop_device((device_send, device_handle));

    Ok(())
}
