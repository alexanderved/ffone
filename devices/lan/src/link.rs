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
use core::util::ClockTime;
use core::util::Element;
use core::util::{ControlFlow, Runnable, Timer};

use core::mueue::*;

const PING_INTERVAL: ClockTime = ClockTime::from_secs(5);
const PONG_INTERVAL: ClockTime = ClockTime::from_secs(10);

pub struct LanLink {
    send: Option<MessageSender<DeviceSystemElementMessage>>,
    info: LanDeviceInfo,

    poller: Poller,
    msg_stream: MessageStream,
    audio_stream: AudioStream,

    ping_timer: Timer,
    pong_timer: Timer,
}

impl LanLink {
    pub fn new(info: LanDeviceInfo) -> error::Result<Self> {
        let mut poller = Poller::new().unwrap();

        let mut msg_stream = MessageStream::new(info.msg_addr).unwrap();
        poller.register_message_stream(&mut msg_stream).unwrap();

        let mut audio_stream = AudioStream::new(info.audio_addr).unwrap();
        poller.register_audio_stream(&mut audio_stream).unwrap();

        msg_stream.push(HostMessage::Connected {
            audio_port: audio_stream.socket().local_addr().unwrap().port(),
        });

        Ok(Self {
            send: None,
            info: info.clone(),

            poller,
            msg_stream,
            audio_stream,

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
        while let Some(audio) = self.audio_stream.pull() {
            self.send(DeviceSystemElementMessage::MuxedAudioReceived(audio));
        }
    }

    fn handle_device_messages(&mut self) {
        while let Some(msg) = self.msg_stream.pull() {
            match msg {
                DeviceMessage::Pong => self.on_pong_received(),
                DeviceMessage::Info { info } => self.on_info_received(info),
            }
        }
    }

    fn ping(&mut self) {
        self.msg_stream.push(HostMessage::Ping);
    }

    fn on_pong_received(&self) {
        self.pong_timer.reset();
    }

    fn on_info_received(&mut self, info: DeviceInfo) {
        self.info.info = info.clone();
        self.send(DeviceSystemElementMessage::LinkedDeviceInfo(info));
    }
}

impl Element for LanLink {
    type Message = DeviceSystemElementMessage;

    fn sender(&self) -> MessageSender<Self::Message> {
        self.send.clone().expect("A device link sender wasn't set")
    }

    fn connect(&mut self, send: MessageSender<Self::Message>) {
        self.send = Some(send);
    }
}

impl Runnable for LanLink {
    fn update(&mut self, flow: &mut ControlFlow) -> error::Result<()> {
        self.handle_ping(flow);

        self.poller
            .poll(&mut self.msg_stream, &mut self.audio_stream)?;
        self.handle_audio();
        self.handle_device_messages();

        Ok(())
    }
}

impl DeviceLink for LanLink {
    fn info(&self) -> DeviceInfo {
        self.info.info()
    }
}
