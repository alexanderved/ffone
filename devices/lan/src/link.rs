use crate::audio_stream::AudioStream;
use crate::message_stream::MessageStream;
use crate::poller::Poller;

use super::*;

use core::device::element::DeviceSystemElementMessage;
use core::device::link::*;
use core::device::*;
use core::error;
use core::util::Element;
use core::util::{ControlFlow, Runnable, Timer};

use core::mueue::*;

use std::time::Duration;

const PING_INTERVAL: Duration = Duration::from_secs(5);
const PONG_INTERVAL: Duration = Duration::from_secs(10);

pub struct LanLink {
    send: Option<MessageSender<DeviceSystemElementMessage>>,
    info: LanDeviceInfo,

    poller: Poller,
    msg_stream: MessageStream,
    audio_stream: Option<AudioStream>,

    ping_timer: Timer,
    pong_timer: Timer,
}

impl LanLink {
    pub fn new(info: LanDeviceInfo) -> error::Result<Self> {
        Ok(Self {
            send: None,
            info: info.clone(),

            poller: Poller::new()?,
            msg_stream: MessageStream::new(info.addr)?,
            audio_stream: None,

            ping_timer: Timer::new(PING_INTERVAL),
            pong_timer: Timer::new(PONG_INTERVAL),
        })
    }

    pub fn on_pong(&mut self) {
        self.pong_timer.restart();
    }

    pub fn on_info_received(&mut self, info: DeviceInfo) {
        self.info.info = info;
    }

    pub fn on_audio_port_received(&mut self, port: u16) {
        if let Some(audio_stream) = self.audio_stream.as_mut() {
            let _ = self.poller.deregister_audio_stream(audio_stream);
        }

        let ip = self.info.addr.ip();
        let mut audio_stream = match AudioStream::new(SocketAddr::from((ip, port))) {
            Ok(audio_stream) => audio_stream,
            Err(_) => {
                self.msg_stream.store(HostMessage::GetAudioPort);
                return;
            }
        };

        let _ = self.poller.register_audio_stream(&mut audio_stream);
        self.audio_stream = Some(audio_stream);
    }
}

impl Element for LanLink {
    type Message = DeviceSystemElementMessage;

    fn sender(&self) -> MessageSender<Self::Message> {
        self.send
            .clone()
            .expect("A device link endpoint wasn't set")
    }

    fn connect(&mut self, send: MessageSender<Self::Message>) {
        self.send = Some(send);
    }
}

impl Runnable for LanLink {
    fn update(&mut self, flow: &mut ControlFlow) -> error::Result<()> {
        if self.ping_timer.is_time_out() {
            self.msg_stream.store(HostMessage::Ping);
        }

        if self.pong_timer.is_time_out() {
            *flow = ControlFlow::Break;

            return Ok(());
        }

        self.poller
            .poll(&mut self.msg_stream, self.audio_stream.as_mut())?;

        while let Some(msg) = self.msg_stream.load() {
            match msg {
                DeviceMessage::Pong => self.on_pong(),
                DeviceMessage::Info { info } => self.on_info_received(info),
                DeviceMessage::AudioPort { port } => self.on_audio_port_received(port),
            }
        }

        while let Some(audio) = self
            .audio_stream
            .as_mut()
            .and_then(|audio_stream| audio_stream.load())
        {
            self.send(DeviceSystemElementMessage::EncodedAudioReceived(audio));
        }

        Ok(())
    }

    fn on_start(&mut self) -> error::Result<()> {
        self.poller.register_message_stream(&mut self.msg_stream)?;
        if let Some(audio_stream) = self.audio_stream.as_mut() {
            self.poller.register_audio_stream(audio_stream)?;
        }

        Ok(())
    }

    fn on_stop(&mut self) -> error::Result<()> {
        self.poller
            .deregister_message_stream(&mut self.msg_stream)?;
        if let Some(audio_stream) = self.audio_stream.as_mut() {
            self.poller.deregister_audio_stream(audio_stream)?;
        }

        Ok(())
    }
}

impl DeviceLink for LanLink {
    fn info(&self) -> DeviceInfo {
        self.info.info()
    }
}

#[cfg(test)]
mod tests {
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

            let packet = link.recv_packet()?;
            let msg = packet.deserialize()?;

            let packet = match msg {
                HostMessage::Ping => NetworkPacket::serialize(&DeviceMessage::Pong),
                HostMessage::GetAudioPort => NetworkPacket::serialize(&DeviceMessage::AudioPort {
                    port: Self::AUDIO_PORT,
                }),
            };

            link.send_packet(&packet?)?;

            Ok(())
        }

        fn on_start(&mut self) -> error::Result<()> {
            let mut link = self.listener.accept()?.0;
            link.set_nonblocking(true)?;
            link.set_nodelay(true)?;

            link.send_packet(&NetworkPacket::serialize(&DeviceMessage::Info {
                info: self.info(),
            })?)?;
            link.send_packet(&NetworkPacket::serialize(&DeviceMessage::AudioPort {
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
}
