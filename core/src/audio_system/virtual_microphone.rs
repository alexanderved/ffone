use super::audio_receiver::*;
use super::element::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtualMicrophoneInfo {
    name: String,
}

pub trait VirtualMicrophone: AudioSink<In = AudioRawData> {
    fn info(&self) -> VirtualMicrophoneInfo;
}
