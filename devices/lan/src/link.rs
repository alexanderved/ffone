#[cfg(test)]
mod tests;

use super::*;

use crate::audio_stream::AudioStream;
use crate::message_stream::MessageStream;
use crate::poller::Poller;

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
