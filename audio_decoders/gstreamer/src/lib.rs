mod gst_context;

use gst_context::GstContext;

use core::audio_system::audio::{
    EncodedAudioBuffer, EncodedAudioHeader, TimestampedRawAudioBuffer,
};
use core::audio_system::element::{AudioFilter, AudioSink, AudioSource, AudioSystemElementMessage};
use core::audio_system::pipeline::audio_decoder::{AudioDecoder, AudioDecoderInfo};
use core::error;
use core::mueue::*;
use core::util::{ControlFlow, Element, Runnable};

pub struct GstDecoder {
    send: MessageSender<AudioSystemElementMessage>,

    input: Option<MessageReceiver<EncodedAudioBuffer>>,
    output: Option<MessageSender<TimestampedRawAudioBuffer>>,

    audio_info: Option<EncodedAudioHeader>,
    context: Option<GstContext>,
}

impl GstDecoder {
    fn update_audio_info(&mut self, info: EncodedAudioHeader) {
        if self.audio_info == Some(info) {
            return;
        }

        self.drain();
        self.context = Some(GstContext::new(info));

        if self.context.is_some() {
            self.audio_info = Some(info);
        }
    }

    fn drain(&self) {
        let Some(context) = self.context.as_ref() else {
            return;
        };

        context.push_eos();
        while !context.is_eos() {
            if context.is_playing_failed() {
                break;
            }

            if let Some(audio) = context.pull() {
                if let Some(output) = self.output.as_ref() {
                    let _ = output.send(audio);
                }
            }
        }
    }
}

impl Runnable for GstDecoder {
    fn update(&mut self, _flow: &mut ControlFlow) -> error::Result<()> {
        let Some(input) = self.input.clone() else {
            return Ok(());
        };

        while let Some(audio) = input.recv() {
            self.update_audio_info(audio.header);

            if let Some(context) = self.context.as_ref() {
                context.push(audio);
            }
        }

        let Some(context) = self.context.as_ref() else {
            return Ok(());
        };

        while let Some(audio) = context.pull() {
            if let Some(output) = self.output.as_ref() {
                let _ = output.send(audio);
            }
        }

        Ok(())
    }

    fn on_start(&mut self) {
        if let Some(context) = self.context.as_ref() {
            context.make_playing();
        }
    }

    fn on_stop(&mut self) {
        if let Some(context) = self.context.as_ref() {
            self.drain();
            context.make_null();
        }
    }
}

impl Element for GstDecoder {
    type Message = AudioSystemElementMessage;

    fn sender(&self) -> MessageSender<Self::Message> {
        self.send.clone()
    }

    fn connect(&mut self, send: MessageSender<Self::Message>) {
        self.send = send;
    }
}

impl AudioSink<EncodedAudioBuffer> for GstDecoder {
    fn input(&self) -> Option<MessageReceiver<EncodedAudioBuffer>> {
        self.input.clone()
    }

    fn set_input(&mut self, input: MessageReceiver<EncodedAudioBuffer>) {
        self.input = Some(input);
    }

    fn unset_input(&mut self) {
        self.input = None;
    }
}

impl AudioSource<TimestampedRawAudioBuffer> for GstDecoder {
    fn output(&self) -> Option<MessageSender<TimestampedRawAudioBuffer>> {
        self.output.clone()
    }

    fn set_output(&mut self, output: MessageSender<TimestampedRawAudioBuffer>) {
        self.output = Some(output);
    }

    fn unset_output(&mut self) {
        self.output = None;
    }
}

impl AudioFilter<EncodedAudioBuffer, TimestampedRawAudioBuffer> for GstDecoder {}

impl AudioDecoder for GstDecoder {
    fn info(&self) -> AudioDecoderInfo {
        AudioDecoderInfo {
            name: "Gstreamer Audio Decoder".to_string(),
        }
    }
}
