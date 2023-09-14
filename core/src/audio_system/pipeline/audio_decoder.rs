use crate::audio_system::{
    audio::{EncodedAudioBuffer, TimestampedRawAudioBuffer},
    element::*,
};
use crate::util::RunnableStateMachine;

pub type AudioDecoderStateMachine = RunnableStateMachine<Box<dyn AudioDecoder>>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AudioDecoderInfo {
    pub name: String,
}

pub trait AudioDecoder: AudioFilter<EncodedAudioBuffer, TimestampedRawAudioBuffer> {
    fn info(&self) -> AudioDecoderInfo;
}

crate::trait_alias!(pub AudioDecoderBuilder:
    AudioSystemElementBuilder<Element = dyn AudioDecoder>);
