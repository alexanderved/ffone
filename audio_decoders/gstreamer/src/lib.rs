mod gst_context;

use gst_context::GstContext;

use core::audio_system::audio::{EncodedAudioBuffer, EncodedAudioInfo, TimestampedRawAudioBuffer};
use core::audio_system::element::{AudioSource, AudioSystemElementMessage};
use core::audio_system::pipeline::audio_decoder::{AudioDecoder, AudioDecoderInfo};
use core::error;
use core::mueue::*;
use core::util::{ControlFlow, Element, Runnable};

pub struct GstDecoder {
    send: MessageSender<AudioSystemElementMessage>,
    output: Option<MessageSender<TimestampedRawAudioBuffer>>,

    context: Option<GstContext>,
}

impl GstDecoder {
    fn drain(&self) {
        let Some(output) = self.output.as_ref() else {
            return;
        };
        let Some(context) = self.context.as_ref() else {
            return;
        };

        context.push_eos();
        while !context.is_eos() {
            if context.is_playing_failed() {
                break;
            }

            if let Some(audio) = context.pull() {
                let _ = output.send(audio);
            }
        }
    }
}

impl Runnable for GstDecoder {
    fn update(&mut self, _flow: &mut ControlFlow) -> error::Result<()> {
        let Some(output) = self.output.as_ref() else {
            return Ok(());
        };
        let Some(context) = self.context.as_ref() else {
            return Ok(());
        };
        if context.is_eos() {
            return Ok(());
        }

        while let Some(audio) = context.pull() {
            let _ = output.send(audio);
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

impl AudioSource<TimestampedRawAudioBuffer> for GstDecoder {
    fn set_output(&mut self, output: MessageSender<TimestampedRawAudioBuffer>) {
        self.output = Some(output);
    }

    fn unset_output(&mut self) {
        self.output = None;
    }
}

impl AudioDecoder for GstDecoder {
    fn info(&self) -> AudioDecoderInfo {
        AudioDecoderInfo {
            name: "Gstreamer Audio Decoder".to_string(),
        }
    }

    fn set_audio_info(&mut self, info: EncodedAudioInfo) {
        self.drain();
        self.context = Some(GstContext::new(info));
    }

    fn enqueue_audio_buffer(&mut self, buf: EncodedAudioBuffer) {
        if let Some(context) = self.context.as_ref() {
            context.push(buf);
        }
    }
}
