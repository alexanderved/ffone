use super::audio_filter::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AudioReceiverInfo {
    name: String,
}

pub trait AudioReceiver: AudioFilter {
    fn info(&self) -> AudioReceiverInfo;
}
