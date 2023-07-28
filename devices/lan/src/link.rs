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

    fn handle_ping(&mut self, flow: &mut ControlFlow) {
        if self.ping_timer.is_time_out() {
            self.ping();
        }

        if self.pong_timer.is_time_out() {
            *flow = ControlFlow::Break;
        }
    }

    fn handle_audio(&mut self) {
        while let Some(audio) = self.audio_stream.as_mut().and_then(AudioStream::pull) {
            self.send(DeviceSystemElementMessage::EncodedAudioReceived(audio));
        }
    }

    fn handle_device_messages(&mut self) {
        while let Some(msg) = self.msg_stream.pull() {
            match msg {
                DeviceMessage::Pong => self.on_pong_received(),
                DeviceMessage::Info { info } => self.on_info_received(info),
                DeviceMessage::StartAudioListener { port, info } => {
                    self.on_start_audio_listener(port, info)
                }
                DeviceMessage::StopAudioListener => self.on_stop_audio_listener(),
            }
        }
    }

    fn ping(&mut self) {
        self.msg_stream.push(HostMessage::Ping);
    }

    fn audio_listener_started(&mut self, port: u16) {
        self.msg_stream
            .push(HostMessage::AudioListenerStarted { port });
    }

    fn on_pong_received(&self) {
        self.pong_timer.restart();
    }

    fn on_info_received(&mut self, info: DeviceInfo) {
        self.info.info = info.clone();
        self.send(DeviceSystemElementMessage::LinkedDeviceInfo(info));
    }

    fn on_start_audio_listener(&mut self, port: u16, info: EncodedAudioInfo) {
        if let Some(mut audio_stream) = self.audio_stream.take() {
            let _ = self.poller.deregister_audio_stream(&mut audio_stream);
        }

        self.audio_stream = AudioStream::new((self.info.addr.ip(), port).into()).ok();
        if let Some(audio_stream) = self.audio_stream.as_mut() {
            let _ = self.poller.register_audio_stream(audio_stream).unwrap();

            let port = audio_stream.socket().local_addr().unwrap().port();
            self.audio_listener_started(port);
        }

        self.send(DeviceSystemElementMessage::AudioInfoReceived(info));
    }

    fn on_stop_audio_listener(&mut self) {
        if let Some(mut audio_stream) = self.audio_stream.take() {
            let _ = self.poller.deregister_audio_stream(&mut audio_stream);
        }
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
        self.handle_ping(flow);

        self.poller
            .poll(&mut self.msg_stream, self.audio_stream.as_mut())?;
        self.handle_audio();
        self.handle_device_messages();

        Ok(())
    }

    fn on_start(&mut self) {
        let _ = self.poller.register_message_stream(&mut self.msg_stream);

        if let Some(audio_stream) = self.audio_stream.as_mut() {
            let _ = self.poller.register_audio_stream(audio_stream);
        }
    }

    fn on_stop(&mut self) {
        let _ = self.poller
            .deregister_message_stream(&mut self.msg_stream);
        if let Some(audio_stream) = self.audio_stream.as_mut() {
            let _ = self.poller.deregister_audio_stream(audio_stream);
        }
    }
}

impl DeviceLink for LanLink {
    fn info(&self) -> DeviceInfo {
        self.info.info()
    }
}
