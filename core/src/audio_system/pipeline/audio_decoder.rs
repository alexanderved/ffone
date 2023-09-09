use crate::audio_system::{
    audio::{EncodedAudioBuffer, EncodedAudioInfo, TimestampedRawAudioBuffer},
    element::*,
};
use crate::util::RunnableStateMachine;

pub type AudioDecoderStateMachine = RunnableStateMachine<Box<dyn AudioDecoder>>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AudioDecoderInfo {
    pub name: String,
}

pub trait AudioDecoder: AudioSource<TimestampedRawAudioBuffer> {
    fn info(&self) -> AudioDecoderInfo;

    fn set_audio_info(&mut self, info: EncodedAudioInfo);
    fn enqueue_audio_buffer(&mut self, buf: EncodedAudioBuffer);

    fn send_eos(&self) {
        if let Some(output) = self.output() {
            let _ = output.send(TimestampedRawAudioBuffer::NULL);
        }
    }
}

crate::trait_alias!(pub AudioDecoderBuilder:
    AudioSystemElementBuilder<Element = dyn AudioDecoder>);
