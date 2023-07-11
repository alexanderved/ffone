use super::element::*;
use crate::util::RunnableStateMachine;
use mueue::Message;

pub type AudioReceiverStateMachine = RunnableStateMachine<Box<dyn AudioReceiver>>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AudioReceiverInfo {
    name: String,
}

pub struct AudioRawData;

impl Message for AudioRawData {}

pub trait AudioReceiver: AudioSource<AudioRawData> {
    fn info(&self) -> AudioReceiverInfo;
}

pub trait AudioReceiverBuilder: AudioSystemElementBuilder<Element = dyn AudioReceiver> {}
