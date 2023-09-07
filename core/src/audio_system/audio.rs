use mueue::*;

use crate::util::ClockTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedAudioBuffer(pub Vec<u8>);

#[repr(i8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum AudioFormat {
    MpegTS,
    Ogg,
    #[default]
    Unspecified,
}

#[repr(i8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum AudioCodec {
    Opus,
    Vorbis,
    #[default]
    Unspecified,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EncodedAudioInfo {
    pub format: AudioFormat,
    pub codec: AudioCodec,
    pub sample_rate: u32,
}

#[repr(i8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum RawAudioFormat {
    #[default]
    U8,

    S16LE,
    S16BE,

    S24LE,
    S24BE,

    S32LE,
    S32BE,

    F32LE,
    F32BE,
}

impl RawAudioFormat {
    pub const fn no_bytes(self) -> usize {
        use RawAudioFormat::*;

        match self {
            U8 => 1,
            S16LE | S16BE => 2,
            S24LE | S24BE => 3,
            S32LE | S32BE | F32LE | F32BE => 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawAudioBuffer {
    data: Vec<u8>,
    format: RawAudioFormat,
}

impl RawAudioBuffer {
    pub fn new(data: Vec<u8>, format: RawAudioFormat) -> Self {
        Self { data, format }
    }

    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    pub fn as_vec(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn as_vec_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    pub fn as_slice(&self) -> &[u8] {
        self.as_vec()
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        self.as_vec_mut()
    }

    pub fn format(&self) -> RawAudioFormat {
        self.format
    }

    pub fn no_samples(&self) -> usize {
        self.as_slice().len() / self.format().no_bytes()
    }
}

impl Message for RawAudioBuffer {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimestampedRawAudioBuffer {
    raw: RawAudioBuffer,

    start: Option<ClockTime>,
    duration: ClockTime,
}

impl TimestampedRawAudioBuffer {
    pub fn new(raw: RawAudioBuffer, start: Option<ClockTime>, duration: ClockTime) -> Self {
        Self {
            raw,
            start,
            duration,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.raw.as_slice()
    }

    pub fn into_raw(self) -> RawAudioBuffer {
        self.raw
    }

    pub fn no_samples(&self) -> usize {
        self.raw.no_samples()
    }

    pub fn start(&self) -> Option<ClockTime> {
        self.start
    }

    pub fn stop(&self) -> Option<ClockTime> {
        self.start.map(|start| start + self.duration)
    }

    pub fn duration(&self) -> ClockTime {
        self.duration
    }

    pub fn sample_duration(&self) -> ClockTime {
        let duration = self.duration();
        let no_samples = self.raw.no_samples() as u64;

        duration / no_samples
    }
}

impl Message for TimestampedRawAudioBuffer {}

pub struct ResizableRawAudioBuffer {
    raw: RawAudioBuffer,
    desired_no_samples: usize,
}

impl ResizableRawAudioBuffer {
    pub fn new(raw: RawAudioBuffer, desired_no_samples: usize) -> Self {
        Self {
            raw,
            desired_no_samples,
        }
    }

    pub fn into_raw(self) -> RawAudioBuffer {
        self.raw
    }

    pub fn no_samples(&self) -> usize {
        self.raw.no_samples()
    }

    pub fn desired_no_samples(&self) -> usize {
        self.desired_no_samples
    }
}

impl Message for ResizableRawAudioBuffer {}
