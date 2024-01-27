extern crate ffi as ffone_ffi;

mod clock;
mod ffi;

use self::ffi::*;
use clock::PAClock;

use core::audio_system::audio::RawAudioBuffer;
use core::audio_system::element::AudioSink;
use core::audio_system::element::AudioSystemElementMessage;
use core::audio_system::pipeline::virtual_microphone::*;
use core::error;
use core::mueue::*;
use core::util::*;

use ffone_ffi::audio_system::queue::RawAudioQueueRC;
use ffone_ffi::rc::ffone_rc_ref;
use ffone_ffi::rc::ffone_rc_unref;

use std::ptr::NonNull;
use std::rc::Rc;

const MAX_PREBUF: usize = 0;

pub struct PAVirtualMicrophone {
    send: MessageSender<AudioSystemElementMessage>,
    input: Option<MessageReceiver<RawAudioBuffer>>,

    queue: RawAudioQueueRC,

    pa_core: NonNull<FFonePACore>,
    pa_stream: *mut FFonePAStream,

    prebuf: usize,
    playing: bool,
}

impl PAVirtualMicrophone {
    pub fn new(send: MessageSender<AudioSystemElementMessage>) -> Option<Self> {
        let queue = RawAudioQueueRC::new()?;
        let pa_core = unsafe { NonNull::new(ffone_pa_core_new()) }?;

        Some(Self {
            send,
            input: None,

            queue,

            pa_core,
            pa_stream: std::ptr::null_mut(),

            prebuf: 0,
            playing: false,
        })
    }
}

impl Runnable for PAVirtualMicrophone {
    fn on_start(&mut self) {
        if !self.pa_stream.is_null() {
            unsafe {
                ffone_rc_unref(self.pa_stream.cast());
            }
        }

        self.pa_stream = unsafe {
            ffone_pa_stream_new(self.pa_core.as_ptr().cast(), self.queue.as_raw())
        };
    }

    fn on_stop(&mut self) {
        unsafe {
            ffone_rc_unref(self.pa_stream.cast());
        }
    }

    fn update(&mut self) -> error::Result<()> {
        let Some(input) = self.input.as_ref() else {
            return Ok(());
        };

        for audio in input.iter() {
            self.prebuf += 1;

            self.queue.push_buffer(audio);
        }

        if self.prebuf > MAX_PREBUF && !self.playing {
            unsafe {
                ffone_pa_stream_play(self.pa_stream);
            }

            self.playing = true;
        }
        
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
            let stream = ffone_rc_ref(self.pa_stream.cast()).cast::<FFonePAStream>();
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
            ffone_rc_unref(self.pa_core.as_ptr().cast());
        }
    }
}
