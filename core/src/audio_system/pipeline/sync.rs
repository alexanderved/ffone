#[cfg(test)]
mod tests;

use crate::error;
use crate::util::{
    Clock, ClockTime, ControlFlow, Element, Runnable, SlaveClock, Timer, OBSERVATIONS_INTERVAL,
};

use crate::audio_system::audio::{ResizableRawAudioBuffer, TimestampedRawAudioBuffer};
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
    output: Option<MessageSender<ResizableRawAudioBuffer>>,

    sys_clock: Arc<dyn Clock>,
    virtual_mic_clock: Option<Rc<dyn SlaveClock>>,
    virtual_mic_clock_update_timer: Timer,

    first_buf_arrival_ts: Option<ClockTime>,
    first_buf_start_ts: Option<ClockTime>,
    buffer_expected_ts: ClockTime,

    queue: VecDeque<TimestampedRawAudioBuffer>,
}

impl Synchronizer {
    pub(in crate::audio_system) fn new(
        send: MessageSender<AudioSystemElementMessage>,
        sys_clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            send,
            input: None,
            output: None,

            sys_clock,
            virtual_mic_clock: None,
            virtual_mic_clock_update_timer: Timer::new(OBSERVATIONS_INTERVAL),

            first_buf_arrival_ts: None,
            first_buf_start_ts: None,
            buffer_expected_ts: ClockTime::ZERO,

            queue: VecDeque::new(),
        }
    }

    pub(in crate::audio_system) fn set_virtual_microphone_clock(
        &mut self,
        virtual_mic_clock: Rc<dyn SlaveClock>,
    ) {
        self.virtual_mic_clock = Some(virtual_mic_clock);
    }

    pub(in crate::audio_system) fn unset_virtual_microphone_clock(&mut self) {
        self.virtual_mic_clock = None;
    }

    fn collect_audio_buffers(&mut self) {
        if let Some(input) = self.input.as_ref() {
            self.queue.extend(input.iter());
        };
    }

    fn process_audio_buffers(&mut self) {
        while let Some(mut ts_buf) = self.queue.pop_front() {
            if ts_buf.no_samples() == 0 {
                continue;
            }
            if ts_buf == TimestampedRawAudioBuffer::NULL {
                self.on_eos();

                continue;
            }

            let elapsed = self.sys_clock.get_time();

            let buf_start_ts = ts_buf.start().unwrap_or(self.buffer_expected_ts);
            let buf_duration = ts_buf.duration();

            let first_buf_arrival_ts = *self.first_buf_arrival_ts.get_or_insert(elapsed);
            let first_buf_start_ts = *self.first_buf_start_ts.get_or_insert(buf_start_ts);

            let desired_play_date = buf_start_ts - first_buf_start_ts + first_buf_arrival_ts;

            if elapsed >= desired_play_date {
                if buf_start_ts < self.buffer_expected_ts {
                    ts_buf.truncate_duration_front(self.buffer_expected_ts - buf_start_ts);
                    self.queue.push_front(ts_buf);

                    continue;
                }
                self.buffer_expected_ts = buf_start_ts + buf_duration;

                let delay = elapsed - desired_play_date;
                let mut real_duration = buf_duration.saturating_sub(delay);

                if let Some(virtual_mic_clock) = self.virtual_mic_clock.as_deref() {
                    let calibration_info = virtual_mic_clock.get_calibration_info();

                    real_duration = real_duration.to_slave_time(calibration_info);
                }

                let sample_duration = ts_buf.sample_duration();
                let desired_no_samples = (real_duration / sample_duration).as_nanos() as usize;
                let buf = ResizableRawAudioBuffer::new(ts_buf.into_raw(), desired_no_samples);

                if let Some(output) = self.output.as_ref() {
                    let _ = output.send(buf);
                }
            } else {
                self.queue.push_front(ts_buf);

                break;
            };
        }
    }

    fn on_eos(&mut self) {
        self.first_buf_arrival_ts = None;
        self.first_buf_start_ts = None;
        self.buffer_expected_ts = ClockTime::ZERO;
    }
}

impl Runnable for Synchronizer {
    fn update(&mut self, _flow: Option<&mut ControlFlow>) -> error::Result<()> {
        if let Some(virtual_mic_clock) = self.virtual_mic_clock.as_deref() {
            if self.virtual_mic_clock_update_timer.is_time_out() {
                virtual_mic_clock.record_observation();
            }
        }

        self.collect_audio_buffers();
        self.process_audio_buffers();

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
    fn input(&self) -> Option<MessageReceiver<TimestampedRawAudioBuffer>> {
        self.input.clone()
    }

    fn set_input(&mut self, input: MessageReceiver<TimestampedRawAudioBuffer>) {
        self.input = Some(input);
    }

    fn unset_input(&mut self) {
        self.input = None;
    }
}

impl AudioSource<ResizableRawAudioBuffer> for Synchronizer {
    fn output(&self) -> Option<MessageSender<ResizableRawAudioBuffer>> {
        self.output.clone()
    }

    fn set_output(&mut self, output: MessageSender<ResizableRawAudioBuffer>) {
        self.output = Some(output);
    }

    fn unset_output(&mut self) {
        self.output = None;
    }
}

impl AudioFilter<TimestampedRawAudioBuffer, ResizableRawAudioBuffer> for Synchronizer {}
