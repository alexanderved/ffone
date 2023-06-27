use super::element::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtualMicrophoneInfo {
    name: String,
}

pub trait VirtualMicrophone: AudioSink {
    fn info(&self) -> VirtualMicrophoneInfo;
}
