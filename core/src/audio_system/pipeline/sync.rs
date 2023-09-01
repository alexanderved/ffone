use crate::error;
use crate::util::{
    Clock, ClockTime, ControlFlow, Element, Runnable, SlaveClock, SystemClock, Timer,
    OBSERVATIONS_INTERVAL,
};

use crate::audio_system::audio::{ShortenableRawAudioBuffer, TimestampedRawAudioBuffer};
use crate::audio_system::element::{
    AudioFilter, AudioSink, AudioSource, AudioSystemElementMessage,
};

use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;

use mueue::*;

pub(in crate::audio_system) struct Synchronizer {
    send: MessageSender<AudioSystemElementMessage>,
    input: Option<MessageReceiver<TimestampedRawAudioBuffer>>,
    output: Option<MessageSender<ShortenableRawAudioBuffer>>,

    sys_clock: Arc<SystemClock>,
    virtual_mic_clock: Option<Rc<dyn SlaveClock>>,
    virtual_mic_clock_update_timer: Timer,

    offset: Option<ClockTime>,
    first_buffer_start_timestamp: Option<ClockTime>,

    queue: VecDeque<TimestampedRawAudioBuffer>,
}

impl Synchronizer {
    pub(in crate::audio_system) fn new(
        send: MessageSender<AudioSystemElementMessage>,
        sys_clock: Arc<SystemClock>,
    ) -> Self {
        Self {
            send,
            input: None,
            output: None,

            sys_clock,
            virtual_mic_clock: None,
            virtual_mic_clock_update_timer: Timer::new(OBSERVATIONS_INTERVAL),

            offset: None,
            first_buffer_start_timestamp: None,

            queue: VecDeque::new(),
        }
    }

    pub(in crate::audio_system) fn set_virtual_microphone_clock(
        &mut self,
        virtual_mic_clock: Rc<dyn SlaveClock>,
    ) {
        self.virtual_mic_clock = Some(virtual_mic_clock);
    }
}

impl Runnable for Synchronizer {
    fn update(&mut self, _flow: &mut ControlFlow) -> error::Result<()> {
        let Some(input) = self.input.as_ref() else {
            return Ok(());
        };
        let Some(output) = self.output.as_ref() else {
            return Ok(());
        };

        if let Some(virtual_mic_clock) = self.virtual_mic_clock.as_deref() {
            if self.virtual_mic_clock_update_timer.is_time_out() {
                virtual_mic_clock.record_observation();
            }
        }

        self.queue.extend(input.iter());
        while let Some(ts_buf) = self.queue.pop_front() {
            let offset = match self.offset {
                Some(offset) => offset,
                None => {
                    self.offset = Some(self.sys_clock.get_time());
                    self.offset.unwrap()
                }
            };
            let first_buffer_start_timestamp = match self.first_buffer_start_timestamp {
                Some(first_buffer_start_timestamp) => first_buffer_start_timestamp,
                None => {
                    self.first_buffer_start_timestamp = Some(ts_buf.start());
                    self.first_buffer_start_timestamp.unwrap()
                }
            };

            let elapsed = self.sys_clock.get_time();
            let desired_play_date = ts_buf.start() + offset - first_buffer_start_timestamp;

            if elapsed >= desired_play_date {
                let mut duration = ClockTime::from_dur(ts_buf.duration());
                let sample_duration = ClockTime::from_dur(ts_buf.sample_duration());

                if let Some(virtual_mic_clock) = self.virtual_mic_clock.as_deref() {
                    let calibration_info = virtual_mic_clock.get_calibration_info();

                    let sys_play_start = elapsed;
                    let sys_play_stop = elapsed + duration;

                    let play_start = sys_play_start.to_slave_time(calibration_info);
                    let play_stop = sys_play_stop.to_slave_time(calibration_info);

                    duration = play_stop - play_start;
                }

                let desired_no_samples = (duration / sample_duration).as_nanos() as usize;
                let buf = ShortenableRawAudioBuffer::new(ts_buf.into_raw(), desired_no_samples);

                let _ = output.send(buf);
            } else {
                self.queue.push_front(ts_buf);
                break;
            };
        }

        Ok(())
    }
}

impl Element for Synchronizer {
    type Message = AudioSystemElementMessage;

    fn sender(&self) -> MessageSender<Self::Message> {
        self.send.clone()
    }

    fn connect(&mut self, send: MessageSender<Self::Message>) {
        self.send = send;
    }
}

impl AudioSink<TimestampedRawAudioBuffer> for Synchronizer {
    fn set_input(&mut self, input: MessageReceiver<TimestampedRawAudioBuffer>) {
        self.input = Some(input);
    }

    fn unset_input(&mut self) {
        self.input = None;
    }
}

impl AudioSource<ShortenableRawAudioBuffer> for Synchronizer {
    fn set_output(&mut self, output: MessageSender<ShortenableRawAudioBuffer>) {
        self.output = Some(output);
    }

    fn unset_output(&mut self) {
        self.output = None;
    }
}

impl AudioFilter<TimestampedRawAudioBuffer, ShortenableRawAudioBuffer> for Synchronizer {}
