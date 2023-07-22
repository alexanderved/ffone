use super::{element::*, RawAudioBuffer};
use crate::util::RunnableStateMachine;

pub type AudioDecoderStateMachine = RunnableStateMachine<Box<dyn AudioDecoder>>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AudioDecoderInfo {
    name: String,
}

pub trait AudioDecoder: AudioSource<RawAudioBuffer> {
    fn info(&self) -> AudioDecoderInfo;
}

crate::trait_alias!(pub AudioDecoderBuilder:
    AudioSystemElementBuilder<Element = dyn AudioDecoder>);
