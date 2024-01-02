extern crate ffi as ffone_ffi;

mod clock;
mod ffi;

use self::ffi::ffone_pa_ctx_get_stream;
use self::ffi::ffone_pa_ctx_new;
use self::ffi::ffone_pa_ctx_update;
use self::ffi::FFonePAContext;
use self::ffi::FFonePAStream;
use clock::PAClock;

use core::audio_system::audio::RawAudioBuffer;
use core::audio_system::element::AudioSink;
use core::audio_system::element::AudioSystemElementMessage;
use core::audio_system::pipeline::virtual_microphone::*;
use core::error;
use core::mueue::*;
use core::util::*;
use std::cell::Cell;

use ffone_ffi::audio_system::queue::RawAudioQueueRC;
use ffone_ffi::rc::ffone_rc_ref;
use ffone_ffi::rc::ffone_rc_unref;

use std::ptr::NonNull;
use std::rc::Rc;

pub struct PAVirtualMicrophone {
    send: MessageSender<AudioSystemElementMessage>,
    input: Option<MessageReceiver<RawAudioBuffer>>,

    queue: RawAudioQueueRC,
    available_duration: Rc<Cell<ClockTime>>,

    pa_ctx: NonNull<FFonePAContext>,

    started: bool,
}

impl PAVirtualMicrophone {
    pub fn new(send: MessageSender<AudioSystemElementMessage>) -> Option<Self> {
        let queue = RawAudioQueueRC::new()?;
        let pa_ctx = unsafe { NonNull::new(ffone_pa_ctx_new(queue.clone().into_raw())) }?;

        Some(Self {
            send,
            input: None,

            queue,
            available_duration: Rc::new(Cell::new(ClockTime::ZERO)),

            pa_ctx,

            started: false,
        })
    }
}

impl Runnable for PAVirtualMicrophone {
    fn update(&mut self) -> error::Result<()> {
        let Some(input) = self.input.as_ref() else {
            return Ok(());
        };

        let audios = input.iter().collect::<Vec<_>>();
        let is_not_empty = !audios.is_empty();

        for audio in audios {
            self.queue.push_buffer(audio);
        }

        if self.queue.no_buffers() >= 3 {
            self.started = true;
        }

        if self.started {
            unsafe { ffone_pa_ctx_update(self.pa_ctx.as_ptr()) };
        }
        
        if is_not_empty {
            dbg!(self.queue.no_bytes(), self.queue.no_buffers());
        }

        self.available_duration.set(ClockTime::ZERO);

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
        let clock: Option<Rc<dyn SlaveClock>> = unsafe {
            let stream = ffone_pa_ctx_get_stream(self.pa_ctx.as_ptr());
            let stream = ffone_rc_ref(stream.cast()).cast::<FFonePAStream>();
            if stream.is_null() {
                None
            } else {
                let clock = PAClock::new(stream);
                Some(Rc::new(SlavedClock::new(clock)))
            }
        };

        clock
    }
}

impl Drop for PAVirtualMicrophone {
    fn drop(&mut self) {
        unsafe {
            ffone_rc_unref(self.pa_ctx.as_ptr().cast());
        }
    }
}
