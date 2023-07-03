use mueue::Message;

use super::element::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AudioReceiverInfo {
    name: String,
}

pub struct AudioRawData;

impl Message for AudioRawData {}

pub trait AudioReceiver: AudioSource<Out = AudioRawData> {
    fn info(&self) -> AudioReceiverInfo;
}
