use crate::*;
use crate::audio_system::*;
use crate::view::*;
use crate::device_link::*;

use std::sync::Arc;

use mueue::{IteratorRun, Message, MessageEndpoint, MessageIterator};

type ViewEndpoint = MessageEndpoint<ViewMessage, ViewControlMessage>;
type AudioSystemEndpoint = MessageEndpoint<AudioSystemMessage, AudioSystemControlMessage>;
type DeviceEndpoint = MessageEndpoint<DeviceMessage, DeviceControlMessage>;

pub enum ControlMessage {
    View(ViewControlMessage),
    AudioSystem(AudioSystemControlMessage),
    Device(DeviceControlMessage),
}

impl Message for ControlMessage {}

pub struct Controller {
    view_end: Option<ViewEndpoint>,
    audio_system_end: Option<AudioSystemEndpoint>,
    device_end: Option<DeviceEndpoint>,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            view_end: None,
            audio_system_end: None,
            device_end: None,
        }
    }

    pub fn connect_view(&mut self, end: ViewEndpoint) {
        self.view_end = Some(end);
    }

    pub fn view_endpoint(&self) -> ViewEndpoint {
        self.view_end
            .clone()
            .expect("A view message endpoint wasn't set")
    }

    pub fn connect_audio_system(&mut self, end: AudioSystemEndpoint) {
        self.audio_system_end = Some(end);
    }

    pub fn audio_system_endpoint(&self) -> AudioSystemEndpoint {
        self.audio_system_end
            .clone()
            .expect("An audio system message endpoint wasn't set")
    }

    pub fn connect_device(&mut self, end: DeviceEndpoint) {
        self.device_end = Some(end);
    }

    pub fn device_endpoint(&self) -> DeviceEndpoint {
        self.device_end
            .clone()
            .expect("A device message endpoint wasn't set")
    }

    pub fn send(&self, msg: ControlMessage) {
        match msg {
            ControlMessage::View(view_msg) => {
                let _ = self.view_endpoint().send(Arc::new(view_msg));
            }
            ControlMessage::AudioSystem(audio_sys_msg) => {
                let _ = self.audio_system_endpoint().send(Arc::new(audio_sys_msg));
            },
            ControlMessage::Device(device_msg) => {
                let _ = self.device_endpoint().send(Arc::new(device_msg));
            }
        }
    }
}

impl Runnable for Controller {
    fn update(&mut self) {
        self.view_endpoint().iter().handle(|_msg| todo!()).run();

        self.audio_system_endpoint()
            .iter()
            .handle(|_msg| todo!())
            .run();
    }
}