use super::element::*;
use super::RawAudioBuffer;
use crate::util::RunnableStateMachine;

pub type VirtualMicrophoneStateMachine = RunnableStateMachine<Box<dyn VirtualMicrophone>>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtualMicrophoneInfo {
    name: String,
}

pub trait VirtualMicrophone: AudioSink<RawAudioBuffer> {
    fn info(&self) -> VirtualMicrophoneInfo;
}

crate::trait_alias!(pub VirtualMicrophoneBuilder:
    AudioSystemElementBuilder<Element = dyn VirtualMicrophone>);
