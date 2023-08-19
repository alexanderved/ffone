#[cfg(test)]
mod tests;

use super::audio::*;
use super::element::*;

use crate::error;
use crate::util::*;

use std::iter;
use std::ops;

use mueue::*;

pub(super) struct AudioDownsampler {
    send: MessageSender<AudioSystemElementMessage>,
    input: Option<MessageReceiver<AudioStreamTask>>,
    output: Option<MessageSender<AudioStreamTask>>,
}

impl AudioDownsampler {
    pub(super) fn new(send: MessageSender<AudioSystemElementMessage>) -> Self {
        Self {
            send,
            input: None,
            output: None,
        }
    }

    fn downsample(&self, audio: RawAudioBuffer, mut rate: f64) -> RawAudioBuffer {
        assert!(
            rate >= 1.0,
            "The downsampling rate must be greater than 1.0"
        );

        let mut downsampled_buf = vec![];

        let mut no_samples = audio.no_samples();
        let mut desired_no_samples = (no_samples as f64 / rate) as usize;

        let mut samples: Vec<Sample> = Vec::with_capacity(rate.ceil() as usize);
        for sample in SampleIter::new(&audio) {
            samples.push(sample);

            let max_rate = rate.ceil() as usize;
            if samples.len() == max_rate {
                let average_sample = samples.drain(..).sum::<Sample>() / max_rate as u8;
                downsampled_buf.extend(average_sample.into_bytes());

                no_samples -= max_rate;
                desired_no_samples -= 1;
                rate = no_samples as f64 / desired_no_samples as f64;
            }
        }

        if !samples.is_empty() {
            downsampled_buf.extend(samples.into_iter().flat_map(|sample| sample.into_bytes()));
        }

        RawAudioBuffer::new(downsampled_buf, audio.format())
    }
}

impl Runnable for AudioDownsampler {
    fn update(&mut self, _flow: &mut ControlFlow) -> error::Result<()> {
        let Some(input) = self.input.as_ref() else {
            return Ok(());
        };
        let Some(output) = self.output.as_ref() else {
            return Ok(());
        };

        for cmd in input.iter() {
            let cmd = match cmd {
                AudioStreamTask::Downsample { audio, rate } => {
                    let new_audio = self.downsample(audio, rate);
                    AudioStreamTask::Play(new_audio)
                }
                cmd => cmd,
            };

            let _ = output.send(cmd);
        }

        Ok(())
    }
}

impl Element for AudioDownsampler {
    type Message = AudioSystemElementMessage;

    fn sender(&self) -> MessageSender<Self::Message> {
        self.send.clone()
    }

    fn connect(&mut self, send: MessageSender<Self::Message>) {
        self.send = send;
    }
}

impl AudioSink<AudioStreamTask> for AudioDownsampler {
    fn set_input(&mut self, input: MessageReceiver<AudioStreamTask>) {
        self.input = Some(input);
    }

    fn unset_input(&mut self) {
        self.input = None;
    }
}

impl AudioSource<AudioStreamTask> for AudioDownsampler {
    fn set_output(&mut self, output: MessageSender<AudioStreamTask>) {
        self.output = Some(output);
    }

    fn unset_output(&mut self) {
        self.input = None;
    }
}

impl AudioFilter<AudioStreamTask, AudioStreamTask> for AudioDownsampler {}

#[derive(Debug)]
enum Sample {
    U8(u8),

    S16LE(i16),
    S16BE(i16),

    S24LE(i32),
    S24BE(i32),

    S32LE(i32),
    S32BE(i32),

    F32LE(f32),
    F32BE(f32),
}

impl Sample {
    fn from_bytes(buf: &[u8], format: RawAudioFormat) -> Self {
        match format {
            RawAudioFormat::U8 => Sample::U8(buf[0]),
            RawAudioFormat::S16LE => {
                let bytes = buf[0..format.no_bytes()]
                    .try_into()
                    .expect("The byte slice is too long");
                Sample::S16LE(i16::from_le_bytes(bytes))
            }
            RawAudioFormat::S16BE => {
                let bytes = buf[0..format.no_bytes()]
                    .try_into()
                    .expect("The byte slice is too long");
                Sample::S16BE(i16::from_be_bytes(bytes))
            }
            RawAudioFormat::S24LE => {
                let mut bytes = [0; 4];
                bytes[0..3].clone_from_slice(&buf[..format.no_bytes()]);
                Sample::S24LE(i32::from_le_bytes(bytes))
            }
            RawAudioFormat::S24BE => {
                let mut bytes = [0; 4];
                bytes[1..4].clone_from_slice(&buf[..format.no_bytes()]);
                Sample::S24BE(i32::from_be_bytes(bytes))
            }
            RawAudioFormat::S32LE => {
                let bytes = buf[0..format.no_bytes()]
                    .try_into()
                    .expect("The byte slice is too long");
                Sample::S32LE(i32::from_le_bytes(bytes))
            }
            RawAudioFormat::S32BE => {
                let bytes = buf[0..format.no_bytes()]
                    .try_into()
                    .expect("The byte slice is too long");
                Sample::S32BE(i32::from_be_bytes(bytes))
            }
            RawAudioFormat::F32LE => {
                let bytes = buf[0..format.no_bytes()]
                    .try_into()
                    .expect("The byte slice is too long");
                Sample::F32LE(f32::from_le_bytes(bytes))
            }
            RawAudioFormat::F32BE => {
                let bytes = buf[0..format.no_bytes()]
                    .try_into()
                    .expect("The byte slice is too long");
                Sample::F32BE(f32::from_be_bytes(bytes))
            }
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            Sample::U8(a) => [a].into(),
            Sample::S16LE(a) => a.to_le_bytes().into(),
            Sample::S16BE(a) => a.to_be_bytes().into(),
            Sample::S24LE(a) => a.to_le_bytes()[0..3].into(),
            Sample::S24BE(a) => a.to_be_bytes()[1..4].into(),
            Sample::S32LE(a) => a.to_le_bytes().into(),
            Sample::S32BE(a) => a.to_be_bytes().into(),
            Sample::F32LE(a) => a.to_le_bytes().into(),
            Sample::F32BE(a) => a.to_be_bytes().into(),
        }
    }
}

impl ops::Add for Sample {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        use Sample::*;
        match (self, rhs) {
            (U8(a), U8(b)) => U8(a + b),

            (S16BE(a), S16BE(b)) => S16BE(a + b),
            (S16LE(a), S16LE(b)) => S16LE(a + b),

            (S24LE(a), S24LE(b)) => S24LE(a + b),
            (S24BE(a), S24BE(b)) => S24BE(a + b),

            (S32LE(a), S32LE(b)) => S32LE(a + b),
            (S32BE(a), S32BE(b)) => S32BE(a + b),

            (F32LE(a), F32LE(b)) => F32LE(a + b),
            (F32BE(a), F32BE(b)) => F32BE(a + b),

            _ => unimplemented!("Adding different sample variats is not supported"),
        }
    }
}

impl ops::Div<u8> for Sample {
    type Output = Self;

    fn div(self, rhs: u8) -> Self::Output {
        use Sample::*;

        match self {
            U8(a) => U8(a / rhs),
            S16LE(a) => S16LE(a / rhs as i16),
            S16BE(a) => S16BE(a / rhs as i16),
            S24LE(a) => S24LE(a / rhs as i32),
            S24BE(a) => S24BE(a / rhs as i32),
            S32LE(a) => S32LE(a / rhs as i32),
            S32BE(a) => S32BE(a / rhs as i32),
            F32LE(a) => F32LE(a / rhs as f32),
            F32BE(a) => F32BE(a / rhs as f32),
        }
    }
}

impl iter::Sum<Sample> for Sample {
    fn sum<I: Iterator<Item = Sample>>(iter: I) -> Self {
        iter.reduce(|a, b| a + b).expect("No samples were supplied")
    }
}

struct SampleIter<'b> {
    buffer: &'b RawAudioBuffer,
    offset: usize,
}

impl<'b> SampleIter<'b> {
    fn new(buffer: &'b RawAudioBuffer) -> Self {
        Self { buffer, offset: 0 }
    }
}

impl iter::Iterator for SampleIter<'_> {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let no_bytes = self.buffer.format().no_bytes();
        if self.offset + no_bytes > self.buffer.as_slice().len() {
            return None;
        }

        let bytes = &self.buffer.as_slice()[self.offset..self.offset + no_bytes];
        let sample = Sample::from_bytes(bytes, self.buffer.format());

        self.offset += no_bytes;

        Some(sample)
    }
}
