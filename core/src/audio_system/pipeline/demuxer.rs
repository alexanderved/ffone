#[cfg(test)]
mod tests;

use std::collections::VecDeque;

use crate::{
    audio_system::{
        audio::{EncodedAudioBuffer, MuxedAudioBuffer},
        element::{AudioSource, AudioSystemElementMessage},
    },
    error,
    util::{Element, Runnable},
};

use mueue::*;

pub struct AudioDemuxer {
    send: MessageSender<AudioSystemElementMessage>,
    output: Option<MessageSender<EncodedAudioBuffer>>,

    muxed_audio: VecDeque<MuxedAudioBuffer>,
}

impl AudioDemuxer {
    pub fn new(send: MessageSender<AudioSystemElementMessage>) -> Self {
        Self {
            send,
            output: None,

            muxed_audio: VecDeque::new(),
        }
    }

    pub fn push(&mut self, buf: MuxedAudioBuffer) {
        self.muxed_audio.push_back(buf);
    }

    fn pull(&mut self) -> Option<EncodedAudioBuffer> {
        self.muxed_audio
            .pop_front()
            .and_then(|buf| buf.try_into().ok())
    }

    fn drain(&mut self) {
        while let Some(audio) = self.pull() {
            if let Some(output) = self.output.as_ref() {
                let _ = output.send(audio);
            };
        }
    }
}

impl Runnable for AudioDemuxer {
    fn update(&mut self) -> error::Result<()> {
        self.drain();

        Ok(())
    }

    fn on_stop(&mut self) {
        self.drain();
    }
}

impl Element for AudioDemuxer {
    type Message = AudioSystemElementMessage;

    fn sender(&self) -> MessageSender<Self::Message> {
        self.send.clone()
    }

    fn connect(&mut self, send: MessageSender<Self::Message>) {
        self.send = send;
    }
}

impl AudioSource<EncodedAudioBuffer> for AudioDemuxer {
    fn output(&self) -> Option<MessageSender<EncodedAudioBuffer>> {
        self.output.clone()
    }

    fn set_output(&mut self, output: MessageSender<EncodedAudioBuffer>) {
        self.output = Some(output);
    }

    fn unset_output(&mut self) {
        self.output = None;
    }
}
