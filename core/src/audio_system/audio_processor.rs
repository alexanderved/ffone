use super::audio_filter::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AudioProcessorInfo {
    name: String,
}

pub trait AudioProcessor: AudioFilter {
    fn info(&self) -> AudioProcessorInfo;
}
