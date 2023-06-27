use super::element::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AudioReceiverInfo {
    name: String,
}

pub trait AudioReceiver: AudioSource {
    fn info(&self) -> AudioReceiverInfo;
}
