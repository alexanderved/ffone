use super::audio_filter::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtualMicrophoneInfo {
    name: String,
}

pub trait VirtualMicrophone: AudioFilter {
    fn info(&self) -> VirtualMicrophoneInfo;
}
