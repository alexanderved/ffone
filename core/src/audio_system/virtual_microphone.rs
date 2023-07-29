use mueue::Message;

use super::audio::RawAudioBuffer;
use super::element::*;
use crate::util::RunnableStateMachine;

pub type VirtualMicrophoneStateMachine = RunnableStateMachine<Box<dyn VirtualMicrophone>>;

pub enum VirtualMicrophoneCommand {
    Flush,
    Play {
        audio: RawAudioBuffer,
        resmaple_rate: f64,
    },
}

impl Message for VirtualMicrophoneCommand {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtualMicrophoneInfo {
    name: String,
}

pub trait VirtualMicrophone: AudioSink<VirtualMicrophoneCommand> {
    fn info(&self) -> VirtualMicrophoneInfo;
    fn set_sample_rate(&mut self, rate: u32);
}

crate::trait_alias!(pub VirtualMicrophoneBuilder:
    AudioSystemElementBuilder<Element = dyn VirtualMicrophone>);
