#[cfg(test)]
mod tests;

use crate::audio_system::audio::*;
use crate::audio_system::element::*;

use crate::error;
use crate::util::*;

use std::iter;
use std::ops;
use std::ptr;

use mueue::*;
use smallvec::SmallVec;

const TEMP_SAMPLE_BUFFER_LEN: usize = 4;

pub(in crate::audio_system) struct AudioResizer {
    send: MessageSender<AudioSystemElementMessage>,
    input: Option<MessageReceiver<ResizableRawAudioBuffer>>,
    output: Option<MessageSender<RawAudioBuffer>>,
}

impl AudioResizer {
    pub(in crate::audio_system) fn new(send: MessageSender<AudioSystemElementMessage>) -> Self {
        Self {
            send,
            input: None,
            output: None,
        }
    }
}

impl Runnable for AudioResizer {
    fn update(&mut self, _flow: &mut ControlFlow) -> error::Result<()> {
        let Some(input) = self.input.as_ref() else {
            return Ok(());
        };

        for audio in input.iter() {
            let no_samples = audio.no_samples();
            let desired_no_samples = audio.desired_no_samples();
            let raw_audio = audio.into_raw();

            let Some(f) = choose_resize_function(no_samples, desired_no_samples) else {
                continue;
            };

            if let Some(output) = self.output.as_ref() {
                let _ = output.send(f(raw_audio, desired_no_samples));
            }
        }

        Ok(())
    }
}

fn choose_resize_function(
    no_samples: usize,
    desired_no_samples: usize,
) -> Option<fn(RawAudioBuffer, usize) -> RawAudioBuffer> {
    if no_samples == 0 || desired_no_samples == 0 {
        None
    } else if desired_no_samples == no_samples {
        Some(|audio, _| audio)
    } else if desired_no_samples <= no_samples * 3 / 4 {
        Some(discard)
    } else if desired_no_samples <= no_samples {
        Some(downsample)
    } else if desired_no_samples >= no_samples * 4 / 3 {
        Some(add_silence)
    } else if desired_no_samples >= no_samples {
        Some(upsample)
    } else {
        None
    }
}

fn discard(mut audio: RawAudioBuffer, desired_no_samples: usize) -> RawAudioBuffer {
    let no_samples = audio.no_samples();
    if no_samples == desired_no_samples {
        return audio;
    }

    debug_assert!(
        desired_no_samples < no_samples,
        "Desired number of samples should be less
            than the current one when performing discarding"
    );

    let no_bytes = audio.format().no_bytes();
    let samples_truncated = no_samples - desired_no_samples;
    vec_truncate_front(audio.as_vec_mut(), samples_truncated * no_bytes);

    audio
}

fn downsample(mut audio: RawAudioBuffer, mut desired_no_samples: usize) -> RawAudioBuffer {
    let mut no_samples = audio.no_samples();
    if no_samples == desired_no_samples {
        return audio;
    }

    debug_assert!(
        desired_no_samples < no_samples,
        "Desired number of samples should be less
            than the current one when performing downsampling"
    );

    let no_bytes = audio.format().no_bytes();
    let mut next_sample_ptr = audio.as_slice_mut().as_mut_ptr();
    let mut temp_samples = SmallVec::<[Sample; TEMP_SAMPLE_BUFFER_LEN]>::new();

    for sample in SampleIter::new(&audio) {
        let min_rate = no_samples / desired_no_samples;
        if min_rate > 1 {
            temp_samples.push(sample);

            if temp_samples.len() == min_rate {
                let average_sample = take_average_sample(temp_samples.drain(..));
                unsafe {
                    ptr::copy_nonoverlapping(
                        average_sample.to_bytes().as_ptr(),
                        next_sample_ptr,
                        no_bytes,
                    );
                }
            }
        }

        if temp_samples.is_empty() {
            unsafe {
                next_sample_ptr = next_sample_ptr.add(no_bytes);
            }

            no_samples -= min_rate;
            desired_no_samples -= 1;
        }
    }

    if !temp_samples.is_empty() {
        let average_sample = take_average_sample(temp_samples.drain(..));
        unsafe {
            ptr::copy_nonoverlapping(
                average_sample.to_bytes().as_ptr(),
                next_sample_ptr,
                no_bytes,
            );
            next_sample_ptr = next_sample_ptr.add(no_bytes);
        }
    }

    let final_len = next_sample_ptr as usize - audio.as_slice().as_ptr() as usize;
    audio.as_vec_mut().truncate(final_len);

    audio
}

fn take_average_sample(sample_iter: impl Iterator<Item = Sample> + ExactSizeIterator) -> Sample {
    let no_sample = sample_iter.len();
    let sample_sum = sample_iter.sum::<Sample>();

    sample_sum / no_sample
}

fn add_silence(mut audio: RawAudioBuffer, desired_no_samples: usize) -> RawAudioBuffer {
    let no_samples = audio.no_samples();
    if no_samples == desired_no_samples {
        return audio;
    }

    debug_assert!(
        desired_no_samples > no_samples,
        "Desired number of samples should be greater
            than the current one when adding silence"
    );

    let no_bytes = audio.format().no_bytes();
    let silence_bytes = (desired_no_samples - no_samples) * no_bytes;

    audio
        .as_vec_mut()
        .extend(iter::repeat(0).take(silence_bytes));

    audio
}

fn upsample(audio: RawAudioBuffer, mut desired_no_samples: usize) -> RawAudioBuffer {
    let mut no_samples = audio.no_samples();

    if no_samples == desired_no_samples {
        return audio;
    }

    debug_assert!(
        desired_no_samples > no_samples,
        "Desired number of samples should be greater
            than the current one when performing downsampling"
    );

    let no_bytes = audio.format().no_bytes();
    let mut new_audio_bytes = Vec::with_capacity(desired_no_samples * no_bytes);

    no_samples -= 1;
    desired_no_samples -= 1;

    for samples_pair in SamplesPairIter::new(&audio) {
        let additional_samples = desired_no_samples / no_samples - 1;

        let first_sample_bytes = &samples_pair[0].to_bytes()[..no_bytes];
        new_audio_bytes.extend_from_slice(first_sample_bytes);
        interpolate_samples(samples_pair, additional_samples, &mut new_audio_bytes);

        no_samples -= 1;
        desired_no_samples -= additional_samples + 1;
    }

    let last_sample_start = audio.len() - no_bytes;
    let last_sample_bytes = &audio.as_slice()[last_sample_start..];
    let last_sample = Sample::from_bytes(last_sample_bytes, audio.format());
    new_audio_bytes.extend(&last_sample.to_bytes()[..no_bytes]);

    RawAudioBuffer::new(new_audio_bytes, audio.format())
}

fn interpolate_samples(
    [first_sample, second_sample]: [Sample; 2],
    additional_samples: usize,
    samples_dst: &mut Vec<u8>,
) {
    let no_bytes = first_sample.no_bytes();
    let denom = additional_samples + 1;
    for num in 1..denom {
        let interpolated_sample = (first_sample + second_sample) * num / denom;
        let interpolated_sample_bytes = &interpolated_sample.to_bytes()[..no_bytes];
        samples_dst.extend_from_slice(interpolated_sample_bytes);
    }
}

impl Element for AudioResizer {
    type Message = AudioSystemElementMessage;

    fn sender(&self) -> MessageSender<Self::Message> {
        self.send.clone()
    }

    fn connect(&mut self, send: MessageSender<Self::Message>) {
        self.send = send;
    }
}

impl AudioSink<ResizableRawAudioBuffer> for AudioResizer {
    fn input(&self) -> Option<MessageReceiver<ResizableRawAudioBuffer>> {
        self.input.clone()    
    }
    
    fn set_input(&mut self, input: MessageReceiver<ResizableRawAudioBuffer>) {
        self.input = Some(input);
    }

    fn unset_input(&mut self) {
        self.input = None;
    }
}

impl AudioSource<RawAudioBuffer> for AudioResizer {
    fn output(&self) -> Option<MessageSender<RawAudioBuffer>> {
        self.output.clone()
    }
    
    fn set_output(&mut self, output: MessageSender<RawAudioBuffer>) {
        self.output = Some(output);
    }

    fn unset_output(&mut self) {
        self.input = None;
    }
}

impl AudioFilter<ResizableRawAudioBuffer, RawAudioBuffer> for AudioResizer {}

#[derive(Debug, Clone, Copy)]
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

    Unspecified,
}

impl Sample {
    const fn no_bytes(&self) -> usize {
        use Sample as S;

        match self {
            S::U8(_) => 1,
            S::S16LE(_) | S::S16BE(_) => 2,
            S::S24LE(_) | S::S24BE(_) => 3,
            S::S32LE(_) | S::S32BE(_) | S::F32LE(_) | S::F32BE(_) => 4,
            S::Unspecified => 0,
        }
    }

    fn from_bytes(buf: &[u8], format: RawAudioFormat) -> Self {
        use RawAudioFormat as R;
        use Sample as S;

        match format {
            R::U8 => S::U8(buf[0]),
            R::S16LE => {
                let bytes = buf[0..format.no_bytes()]
                    .try_into()
                    .expect("The byte slice is too long");
                S::S16LE(i16::from_le_bytes(bytes))
            }
            R::S16BE => {
                let bytes = buf[0..format.no_bytes()]
                    .try_into()
                    .expect("The byte slice is too long");
                S::S16BE(i16::from_be_bytes(bytes))
            }
            R::S24LE => {
                let mut bytes = [0; 4];
                bytes[0..3].clone_from_slice(&buf[..format.no_bytes()]);
                S::S24LE(i32::from_le_bytes(bytes))
            }
            R::S24BE => {
                let mut bytes = [0; 4];
                bytes[1..4].clone_from_slice(&buf[..format.no_bytes()]);
                S::S24BE(i32::from_be_bytes(bytes))
            }
            R::S32LE => {
                let bytes = buf[0..format.no_bytes()]
                    .try_into()
                    .expect("The byte slice is too long");
                S::S32LE(i32::from_le_bytes(bytes))
            }
            R::S32BE => {
                let bytes = buf[0..format.no_bytes()]
                    .try_into()
                    .expect("The byte slice is too long");
                S::S32BE(i32::from_be_bytes(bytes))
            }
            R::F32LE => {
                let bytes = buf[0..format.no_bytes()]
                    .try_into()
                    .expect("The byte slice is too long");
                S::F32LE(f32::from_le_bytes(bytes))
            }
            R::F32BE => {
                let bytes = buf[0..format.no_bytes()]
                    .try_into()
                    .expect("The byte slice is too long");
                S::F32BE(f32::from_be_bytes(bytes))
            }
            R::Unspecified => S::Unspecified,
        }
    }

    fn to_bytes(&self) -> [u8; 4] {
        use Sample as S;

        let mut bytes = [0; 4];
        match self {
            S::U8(a) => bytes[0..1].clone_from_slice(&[*a]),
            S::S16LE(a) => bytes[0..2].clone_from_slice(&a.to_le_bytes()),
            S::S16BE(a) => bytes[0..2].clone_from_slice(&a.to_be_bytes()),
            S::S24LE(a) => bytes[0..3].clone_from_slice(&a.to_le_bytes()[0..3]),
            S::S24BE(a) => bytes[0..3].clone_from_slice(&a.to_be_bytes()[1..4]),
            S::S32LE(a) => bytes[0..4].clone_from_slice(&a.to_le_bytes()),
            S::S32BE(a) => bytes[0..4].clone_from_slice(&a.to_be_bytes()),
            S::F32LE(a) => bytes[0..4].clone_from_slice(&a.to_le_bytes()),
            S::F32BE(a) => bytes[0..4].clone_from_slice(&a.to_be_bytes()),
            S::Unspecified => {}
        }

        bytes
    }
}

impl ops::Add for Sample {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        use Sample as S;

        match (self, rhs) {
            (S::U8(a), S::U8(b)) => S::U8(a + b),

            (S::S16BE(a), S::S16BE(b)) => S::S16BE(a + b),
            (S::S16LE(a), S::S16LE(b)) => S::S16LE(a + b),

            (S::S24LE(a), S::S24LE(b)) => S::S24LE(a + b),
            (S::S24BE(a), S::S24BE(b)) => S::S24BE(a + b),

            (S::S32LE(a), S::S32LE(b)) => S::S32LE(a + b),
            (S::S32BE(a), S::S32BE(b)) => S::S32BE(a + b),

            (S::F32LE(a), S::F32LE(b)) => S::F32LE(a + b),
            (S::F32BE(a), S::F32BE(b)) => S::F32BE(a + b),

            _ => unimplemented!("Adding different sample variats is not supported"),
        }
    }
}

impl ops::Mul<usize> for Sample {
    type Output = Self;

    fn mul(self, rhs: usize) -> Self::Output {
        use Sample as S;

        match self {
            S::U8(a) => S::U8(a * rhs as u8),
            S::S16LE(a) => S::S16LE(a * rhs as i16),
            S::S16BE(a) => S::S16BE(a * rhs as i16),
            S::S24LE(a) => S::S24LE(a * rhs as i32),
            S::S24BE(a) => S::S24BE(a * rhs as i32),
            S::S32LE(a) => S::S32LE(a * rhs as i32),
            S::S32BE(a) => S::S32BE(a * rhs as i32),
            S::F32LE(a) => S::F32LE(a * rhs as f32),
            S::F32BE(a) => S::F32BE(a * rhs as f32),
            S::Unspecified => S::Unspecified,
        }
    }
}

impl ops::Div<usize> for Sample {
    type Output = Self;

    fn div(self, rhs: usize) -> Self::Output {
        use Sample as S;

        match self {
            S::U8(a) => S::U8(a / rhs as u8),
            S::S16LE(a) => S::S16LE(a / rhs as i16),
            S::S16BE(a) => S::S16BE(a / rhs as i16),
            S::S24LE(a) => S::S24LE(a / rhs as i32),
            S::S24BE(a) => S::S24BE(a / rhs as i32),
            S::S32LE(a) => S::S32LE(a / rhs as i32),
            S::S32BE(a) => S::S32BE(a / rhs as i32),
            S::F32LE(a) => S::F32LE(a / rhs as f32),
            S::F32BE(a) => S::F32BE(a / rhs as f32),
            S::Unspecified => S::Unspecified,
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

struct SamplesPairIter<'b> {
    buffer: &'b RawAudioBuffer,
    offset: usize,
}

impl<'b> SamplesPairIter<'b> {
    fn new(buffer: &'b RawAudioBuffer) -> Self {
        Self { buffer, offset: 0 }
    }
}

impl iter::Iterator for SamplesPairIter<'_> {
    type Item = [Sample; 2];

    fn next(&mut self) -> Option<Self::Item> {
        let no_bytes = self.buffer.format().no_bytes();
        if self.offset + no_bytes * 2 > self.buffer.as_slice().len() {
            return None;
        }

        let first_sample_start = self.offset;
        let first_sample_end = self.offset + no_bytes;

        let first_bytes = &self.buffer.as_slice()[first_sample_start..first_sample_end];
        let first_sample = Sample::from_bytes(first_bytes, self.buffer.format());

        let second_sample_start = self.offset + no_bytes;
        let second_sample_end = self.offset + no_bytes * 2;

        let second_bytes = &self.buffer.as_slice()[second_sample_start..second_sample_end];
        let second_sample = Sample::from_bytes(second_bytes, self.buffer.format());

        self.offset += no_bytes;

        Some([first_sample, second_sample])
    }
}
