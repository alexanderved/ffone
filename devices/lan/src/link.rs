use crate::connection::TcpConnection;

use super::*;

use core::device::element::DeviceSystemElementMessage;
use core::device::link::*;
use core::device::*;
use core::error;
use core::util::Element;
use core::util::{ControlFlow, Runnable, Timer};

use core::mueue::*;

use std::time::Duration;

pub struct LanLink {
    send: Option<MessageSender<DeviceSystemElementMessage>>,

    info: LanDeviceInfo,
    audio_addr: Option<SocketAddr>,

    link: TcpConnection,

    ping_timer: Timer,
    pong_timer: Timer,
}

impl LanLink {
    pub fn new(info: LanDeviceInfo) -> error::Result<Self> {
        let link = TcpConnection::new(info.addr)?;

        Ok(Self {
            send: None,

            info: info.clone(),
            audio_addr: None,

            link,

            ping_timer: Timer::new(Duration::from_secs(1)),
            pong_timer: Timer::new(Duration::from_secs(5)),
        })
    }

    pub fn ping_device(&mut self) -> error::Result<()> {
        const PING_TIMEOUT: Duration = Duration::from_millis(100);
        const PING_RETRIES: usize = 10;

        self.link
            .send(&HostMessage::Ping, Some(PING_TIMEOUT), Some(PING_RETRIES))?;

        Ok(())
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
            if self.ping_device().is_err() {
                *flow = ControlFlow::Break;

                return Ok(());
            }
        }

        if self.pong_timer.is_time_out() {
            *flow = ControlFlow::Break;

            return Ok(());
        }

        self.link.recv_to_buf(Some(Duration::from_millis(0)))?;
        self.link.filter_received_messages(|msg| {
            match msg {
                DeviceMessage::Info { info } => self.info.info = info,
                DeviceMessage::AudioPort { port } => {
                    let ip = self.info.addr.ip();

                    self.audio_addr = Some(SocketAddr::from((ip, port)));
                }
                DeviceMessage::Pong => {
                    self.pong_timer.restart();
                }
                msg => return Some(msg),
            }

            None
        });

        Ok(())
    }
}

impl DeviceLink for LanLink {
    fn info(&self) -> DeviceInfo {
        self.info.info()
    }

    fn audio_address(&self) -> Option<SocketAddr> {
        self.audio_addr.clone()
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
                HostMessage::GetInfo => {
                    NetworkPacket::serialize(&DeviceMessage::Info { info: self.info() })
                }
                HostMessage::GetAudioPort => NetworkPacket::serialize(&DeviceMessage::AudioPort {
                    port: self.audio_port(),
                }),
                HostMessage::Ping => NetworkPacket::serialize(&DeviceMessage::Pong),
                _ => return Ok(()),
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
            /* link.send_packet(&NetworkPacket::serialize(&DeviceMessage::AudioPort {
                port: self.audio_port(),
            })?)?; */

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

    fn create_link(port: u16) -> error::Result<RunnableStateMachine<LanLink>> {
        let (link_send, _link_recv) = unidirectional_queue();
        let mut link = LanLink::new(LanDeviceInfo::new(
            "fake",
            (Ipv4Addr::LOCALHOST, port).into(),
        ))?;
        link.connect(link_send);
        let link = RunnableStateMachine::new_running(link).map_err(|(_, err)| err)?;

        Ok(link)
    }

    #[test]
    fn test_get_info() -> error::Result<()> {
        let device_port = 31707;
        let (device_send, device_handle) = run_device("fake", device_port)?;
        let mut link = create_link(device_port)?;

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
    fn test_get_audio_address() -> error::Result<()> {
        let device_port = 31708;
        let (device_send, device_handle) = run_device("fake", device_port)?;
        let mut link = create_link(device_port)?;

        stop_device((device_send, device_handle));

        let mut addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0));
        while let Some(_) = link.proceed() {
            if let Some(address) = link.as_runnable_mut().audio_address() {
                addr = address;
                break;
            }
        }
        link.stop()?;

        assert_eq!(
            addr,
            SocketAddr::from((Ipv4Addr::LOCALHOST, FakeDevice::AUDIO_PORT))
        );

        Ok(())
    }
}
