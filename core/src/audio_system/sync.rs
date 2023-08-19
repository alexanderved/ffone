use std::collections::VecDeque;
use std::time::{Duration, Instant};

use mueue::*;

use crate::error;
use crate::util::{ControlFlow, Element, Runnable};

use super::audio::{AudioStreamTask, Timestamp, TimestampedRawAudioBuffer};
use super::element::{AudioFilter, AudioSink, AudioSource, AudioSystemElementMessage};

const MIN_DELAY: Duration = Duration::from_millis(20);
const MAX_DELAY: Duration = Duration::from_millis(40);

pub(super) struct Synchronizer {
    send: MessageSender<AudioSystemElementMessage>,
    input: Option<MessageReceiver<TimestampedRawAudioBuffer>>,
    output: Option<MessageSender<AudioStreamTask>>,

    base: Option<Instant>,
    offset: Option<Timestamp>,
    queue: VecDeque<TimestampedRawAudioBuffer>,
}

impl Synchronizer {
    pub(super) fn new(send: MessageSender<AudioSystemElementMessage>) -> Self {
        Self {
            send,
            input: None,
            output: None,

            base: None,
            offset: None,
            queue: VecDeque::new(),
        }
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

        self.queue.extend(input.iter());
        while let Some(ts_buf) = self.queue.pop_front() {
            let base = match self.base {
                Some(base) => base,
                None => {
                    self.base = Some(Instant::now());
                    self.base.unwrap()
                }
            };
            let offset = match self.offset {
                Some(offset) => offset,
                None => {
                    self.offset = Some(ts_buf.start());
                    self.offset.unwrap()
                }
            };

            let play_time = ts_buf.start().as_dur() - offset.as_dur();
            let cmd = if base.elapsed() - play_time >= MAX_DELAY {
                self.base = None;
                self.offset = None;
                self.queue.push_front(ts_buf);

                AudioStreamTask::Flush
            } else if base.elapsed() - play_time >= MIN_DELAY {
                let delay = base.elapsed() - play_time;
                let duration = ts_buf.duration();
                let rate = 1.0 / (1.0 - delay.as_nanos() as f64 / duration.as_nanos() as f64);

                AudioStreamTask::Downsample {
                    audio: ts_buf.into_raw(),
                    rate,
                }
            } else if play_time <= base.elapsed() {
                AudioStreamTask::Play(ts_buf.into_raw())
            } else {
                self.queue.push_front(ts_buf);
                break;
            };

            let _ = output.send(cmd);
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

impl AudioSource<AudioStreamTask> for Synchronizer {
    fn set_output(&mut self, output: MessageSender<AudioStreamTask>) {
        self.output = Some(output);
    }

    fn unset_output(&mut self) {
        self.output = None;
    }
}

impl AudioFilter<TimestampedRawAudioBuffer, AudioStreamTask> for Synchronizer {}