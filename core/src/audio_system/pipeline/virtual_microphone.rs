use crate::audio_system::audio::*;
use crate::audio_system::element::*;
use crate::util::RunnableStateMachine;
use crate::util::SlaveClock;

use std::rc::Rc;

pub type VirtualMicrophoneStateMachine = RunnableStateMachine<Box<dyn VirtualMicrophone>>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtualMicrophoneInfo {
    pub name: String,
}

pub trait VirtualMicrophone: AudioSink<RawAudioBuffer> {
    fn info(&self) -> VirtualMicrophoneInfo;

    fn provide_clock(&self) -> Option<Rc<dyn SlaveClock>> {
        None
    }
}

crate::trait_alias!(pub VirtualMicrophoneBuilder:
    AudioSystemElementBuilder<Element = dyn VirtualMicrophone>);
