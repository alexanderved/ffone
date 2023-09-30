extern crate ffi as ffone_ffi;

mod clock;
mod ffi;

use clock::PAClock;
use self::ffi::ffone_pa_ctx_update;
use self::ffi::FFonePAContext;
use self::ffi::FFonePAStream;
use self::ffi::ffone_pa_ctx_get_stream;
use self::ffi::ffone_pa_ctx_new;

use core::audio_system::audio::RawAudioBuffer;
use core::audio_system::element::AudioSink;
use core::audio_system::element::AudioSystemElementMessage;
use core::audio_system::pipeline::virtual_microphone::*;
use core::error;
use core::mueue::*;
use core::util::*;

use ffone_ffi::rc::ffone_rc_ref;
use ffone_ffi::rc::ffone_rc_unref;
use ffone_ffi::audio_system::queue::RawAudioQueueRC;

use std::rc::Rc;
use std::ptr::NonNull;

pub struct PAVirtualMicrophone {
    send: MessageSender<AudioSystemElementMessage>,
    input: Option<MessageReceiver<RawAudioBuffer>>,

    pub queue: RawAudioQueueRC,
    pa_ctx: NonNull<FFonePAContext>,
}

impl PAVirtualMicrophone {
    pub fn new(send: MessageSender<AudioSystemElementMessage>) -> Option<Self> {
        let queue = RawAudioQueueRC::new()?;
        let pa_ctx = unsafe {
            NonNull::new(ffone_pa_ctx_new(queue.clone().into_raw()))
        }?;

        Some(Self {
            send,
            input: None,

            queue,
            pa_ctx,
        })
    }
}

impl Runnable for PAVirtualMicrophone {
    fn update(&mut self, _flow: Option<&mut ControlFlow>) -> error::Result<()> {
        let Some(input) = self.input.as_ref() else {
            return Ok(());
        };

        for audio in input.iter() {
            self.queue.push_buffer(audio);
        }

        unsafe {
            ffone_pa_ctx_update(self.pa_ctx.as_ptr(), 1)
        };

        Ok(())
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
    fn input(&self) -> Option<MessageReceiver<RawAudioBuffer>> {
        self.input.clone()
    }

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

    fn provide_clock(&self) -> Option<Rc<dyn SlaveClock>> {
        unsafe {
            let stream = ffone_pa_ctx_get_stream(self.pa_ctx.as_ptr());
            let stream = ffone_rc_ref(stream.cast()).cast::<FFonePAStream>();
            if stream.is_null() {
                return None;
            }

            let clock = PAClock::new(stream);
            Some(Rc::new(SlavedClock::new(clock)))
        }
    }
}

impl Drop for PAVirtualMicrophone {
    fn drop(&mut self) {
        unsafe {
            ffone_rc_unref(self.pa_ctx.as_ptr().cast());
        }
    }
}