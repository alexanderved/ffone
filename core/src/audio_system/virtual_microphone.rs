use super::audio_receiver::*;
use super::element::*;
use crate::util::RunnableStateMachine;

pub type VirtualMicrophoneStateMachine = RunnableStateMachine<Box<dyn VirtualMicrophone>>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtualMicrophoneInfo {
    name: String,
}

pub trait VirtualMicrophone: AudioSink<AudioRawData> {
    fn info(&self) -> VirtualMicrophoneInfo;
}

crate::trait_alias!(pub VirtualMicrophoneBuilder:
    AudioSystemElementBuilder<Element = dyn VirtualMicrophone>);
