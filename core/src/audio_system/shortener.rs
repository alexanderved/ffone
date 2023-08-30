#[cfg(test)]
mod tests;

use super::audio::*;
use super::element::*;

use crate::error;
use crate::util::*;

use std::cell::UnsafeCell;
use std::iter;
use std::ops;
use std::ptr;

use mueue::*;

pub(super) struct AudioShortener {
    send: MessageSender<AudioSystemElementMessage>,
    input: Option<MessageReceiver<AudioShortenerTask>>,
    output: Option<MessageSender<RawAudioBuffer>>,

    temp_samples: UnsafeCell<Vec<Sample>>,
}

impl AudioShortener {
    pub(super) fn new(send: MessageSender<AudioSystemElementMessage>) -> Self {
        Self {
            send,
            input: None,
            output: None,

            temp_samples: UnsafeCell::new(vec![]),
        }
    }

    fn downsample(&self, mut audio: RawAudioBuffer, mut rate: f64) -> RawAudioBuffer {
        if (rate - 1.0).abs() <= f64::EPSILON {
            return audio;
        }

        assert!(rate > 1.0, "The downsampling rate must be greater than 1.0");

        let no_bytes = audio.format().no_bytes();
        let mut no_samples = audio.no_samples();
        let mut desired_no_samples = (no_samples as f64 / rate) as usize;

        let mut next_sample_ptr = audio.as_slice_mut().as_mut_ptr();

        let mut bytes = [0; 4];
        let temp_samples = unsafe { &mut *self.temp_samples.get() };

        for sample in SampleIter::new(&audio) {
            let min_rate = rate.floor() as usize;
            if min_rate > 1 {
                temp_samples.push(sample);

                if temp_samples.len() == min_rate {
                    let sample_sum = temp_samples.drain(..).sum::<Sample>();
                    let average_sample = sample_sum / min_rate as u8;

                    average_sample.copy_bytes_into(&mut bytes);
                    unsafe {
                        ptr::copy_nonoverlapping(bytes.as_ptr(), next_sample_ptr, no_bytes);
                    }
                }
            }

            if temp_samples.is_empty() {
                unsafe {
                    next_sample_ptr = next_sample_ptr.add(no_bytes);
                }

                no_samples -= min_rate;
                desired_no_samples -= 1;
                rate = no_samples as f64 / desired_no_samples as f64;
            }
        }

        if !temp_samples.is_empty() {
            let samples_len = temp_samples.len() as u8;
            let average_sample = temp_samples.drain(..).sum::<Sample>() / samples_len;

            average_sample.copy_bytes_into(&mut bytes);
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), next_sample_ptr, no_bytes);
                next_sample_ptr = next_sample_ptr.add(no_bytes);
            }
        }

        let final_len = next_sample_ptr as usize - audio.as_slice().as_ptr() as usize;
        audio.as_vec_mut().truncate(final_len);

        audio
    }

    fn discard(&self, mut audio: RawAudioBuffer, no_samples: usize) -> Option<RawAudioBuffer> {
        let no_bytes = audio.format().no_bytes();
        vec_truncate_front(audio.as_vec_mut(), no_samples * no_bytes);

        if audio.len() > 0 {
            return Some(audio);
        }

        None
    }
}

impl Runnable for AudioShortener {
    fn update(&mut self, _flow: &mut ControlFlow) -> error::Result<()> {
        let Some(input) = self.input.as_ref() else {
            return Ok(());
        };
        let Some(output) = self.output.as_ref() else {
            return Ok(());
        };

        for cmd in input.iter() {
            let new_audio = match cmd {
                AudioShortenerTask::Downsample { audio, rate } => self.downsample(audio, rate),
                AudioShortenerTask::Discard { audio, no_samples } => {
                    match self.discard(audio, no_samples) {
                        Some(new_audio) => new_audio,
                        None => continue,
                    }
                }
            };

            let _ = output.send(new_audio);
        }

        Ok(())
    }
}

impl Element for AudioShortener {
    type Message = AudioSystemElementMessage;

    fn sender(&self) -> MessageSender<Self::Message> {
        self.send.clone()
    }

    fn connect(&mut self, send: MessageSender<Self::Message>) {
        self.send = send;
    }
}

impl AudioSink<AudioShortenerTask> for AudioShortener {
    fn set_input(&mut self, input: MessageReceiver<AudioShortenerTask>) {
        self.input = Some(input);
    }

    fn unset_input(&mut self) {
        self.input = None;
    }
}

impl AudioSource<RawAudioBuffer> for AudioShortener {
    fn set_output(&mut self, output: MessageSender<RawAudioBuffer>) {
        self.output = Some(output);
    }

    fn unset_output(&mut self) {
        self.input = None;
    }
}

impl AudioFilter<AudioShortenerTask, RawAudioBuffer> for AudioShortener {}

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

    fn copy_bytes_into(&self, bytes: &mut [u8]) {
        match self {
            Sample::U8(a) => bytes[0..1].clone_from_slice(&[*a]),
            Sample::S16LE(a) => bytes[0..2].clone_from_slice(&a.to_le_bytes()),
            Sample::S16BE(a) => bytes[0..2].clone_from_slice(&a.to_be_bytes()),
            Sample::S24LE(a) => bytes[0..3].clone_from_slice(&a.to_le_bytes()[0..3]),
            Sample::S24BE(a) => bytes[0..3].clone_from_slice(&a.to_be_bytes()[1..4]),
            Sample::S32LE(a) => bytes[0..4].clone_from_slice(&a.to_le_bytes()),
            Sample::S32BE(a) => bytes[0..4].clone_from_slice(&a.to_be_bytes()),
            Sample::F32LE(a) => bytes[0..4].clone_from_slice(&a.to_le_bytes()),
            Sample::F32BE(a) => bytes[0..4].clone_from_slice(&a.to_be_bytes()),
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
