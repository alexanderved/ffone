use crate::audio_system::*;
use crate::util::*;
use crate::view::*;
use crate::*;

use mueue::{Message, MessageEndpoint};

type ViewEndpoint = MessageEndpoint<ViewMessage, ViewControlMessage>;
type AudioSystemEndpoint = MessageEndpoint<AudioSystemMessage, AudioSystemControlMessage>;

#[non_exhaustive]
pub enum ControlMessage {
    View(ViewControlMessage),
    AudioSystem(AudioSystemControlMessage),
}

impl Message for ControlMessage {}

pub struct Controller {
    view_end: Option<ViewEndpoint>,
    audio_system_end: Option<AudioSystemEndpoint>,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            view_end: None,
            audio_system_end: None,
        }
    }

    pub fn view_endpoint(&self) -> ViewEndpoint {
        self.view_end
            .clone()
            .expect("A view message endpoint wasn't set")
    }

    pub fn connect_view(&mut self, end: ViewEndpoint) {
        self.view_end = Some(end);
    }

    pub fn audio_system_endpoint(&self) -> AudioSystemEndpoint {
        self.audio_system_end
            .clone()
            .expect("An audio system message endpoint wasn't set")
    }

    pub fn connect_audio_system(&mut self, end: AudioSystemEndpoint) {
        self.audio_system_end = Some(end);
    }

    pub fn send(&self, msg: ControlMessage) {
        match msg {
            ControlMessage::View(view_msg) => {
                let _ = self.view_endpoint().send(view_msg);
            }
            ControlMessage::AudioSystem(audio_sys_msg) => {
                let _ = self.audio_system_endpoint().send(audio_sys_msg);
            }
        }
    }
}

impl Runnable for Controller {
    fn update(&mut self, _flow: &mut ControlFlow) -> error::Result<()> {
        self.view_endpoint().iter().for_each(|_msg| todo!());

        self.audio_system_endpoint().iter().for_each(|_msg| todo!());

        todo!()
    }
}
