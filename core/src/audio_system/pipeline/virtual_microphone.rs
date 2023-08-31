use crate::audio_system::audio::*;
use crate::audio_system::element::*;
use crate::util::RunnableStateMachine;

pub type VirtualMicrophoneStateMachine = RunnableStateMachine<Box<dyn VirtualMicrophone>>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtualMicrophoneInfo {
    pub name: String,
}

pub trait VirtualMicrophone: AudioSink<RawAudioBuffer> {
    fn info(&self) -> VirtualMicrophoneInfo;
    fn set_sample_rate(&mut self, rate: u32);
}

crate::trait_alias!(pub VirtualMicrophoneBuilder:
    AudioSystemElementBuilder<Element = dyn VirtualMicrophone>);
