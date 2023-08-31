use core::audio_system::audio::RawAudioBuffer;
use core::audio_system::element::AudioSink;
use core::audio_system::element::AudioSystemElementMessage;
use core::audio_system::queue::RawAudioQueue;
use core::audio_system::virtual_microphone::*;
use core::error;
use core::mueue::*;
use core::util::*;

extern crate ffi;

extern "C" {
    #[allow(improper_ctypes)]
    pub fn cmain(queue: *mut RawAudioQueue);
}

pub struct PAVirtualMicrophone {
    send: MessageSender<AudioSystemElementMessage>,
    input: Option<MessageReceiver<RawAudioBuffer>>,
}

impl Runnable for PAVirtualMicrophone {
    fn update(&mut self, _flow: &mut ControlFlow) -> error::Result<()> {
        todo!()
    }
}

impl Element for PAVirtualMicrophone {
    type Message = AudioSystemElementMessage;

    fn sender(&self) -> MessageSender<Self::Message> {
        self.send.clone()
    }

    fn connect(&mut self, send: MessageSender<Self::Message>) {
        self.send = send;
    }
}

impl AudioSink<RawAudioBuffer> for PAVirtualMicrophone {
    fn set_input(&mut self, input: MessageReceiver<RawAudioBuffer>) {
        self.input = Some(input);
    }

    fn unset_input(&mut self) {
        self.input = None;
    }
}

impl VirtualMicrophone for PAVirtualMicrophone {
    fn info(&self) -> VirtualMicrophoneInfo {
        VirtualMicrophoneInfo {
            name: "Pulseaudio Virtual Microphone".to_string(),
        }
    }

    fn set_sample_rate(&mut self, rate: u32) {
        todo!()
    }
}
