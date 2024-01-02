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

const QUEUE_MAX_DURATION: ClockTime = ClockTime::from_micros(208310);

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
        let queue = RawAudioQueueRC::new(QUEUE_MAX_DURATION)?;
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

        /* if is_not_empty {
            let prev_dur = self.available_duration.get();
            let curr_dur = self.queue.duration();

            dbg!(prev_dur, curr_dur, curr_dur - prev_dur);
        } */

        if self.started {
            unsafe { ffone_pa_ctx_update(self.pa_ctx.as_ptr(), 0) };
        }

        
        let available_duration = self.queue.available_duration();
        
        if is_not_empty {
            dbg!(self.queue.no_bytes(), self.queue.no_buffers());
        }

        //if available_duration >= QUEUE_MAX_DURATION / 2 {
            self.available_duration.set(available_duration);
        /* } else {
            self.available_duration.set(ClockTime::ZERO);
        } */

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

    fn provide_statistics(&self) -> VirtualMicrophoneStatistics {
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

        VirtualMicrophoneStatistics::new(clock, Rc::clone(&self.available_duration))
    }
}

impl Drop for PAVirtualMicrophone {
    fn drop(&mut self) {
        unsafe {
            ffone_rc_unref(self.pa_ctx.as_ptr().cast());
        }
    }
}
