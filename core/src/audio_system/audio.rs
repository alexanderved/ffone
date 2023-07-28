use std::time::Duration;

use mueue::*;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawAudioFormat {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawAudioBuffer {
    data: Vec<u8>,
    format: RawAudioFormat,
}

impl RawAudioBuffer {
    pub fn new(data: Vec<u8>, format: RawAudioFormat) -> Self {
        Self { data, format }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

impl Message for RawAudioBuffer {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(Duration);

impl Timestamp {
    pub fn new(dur: Duration) -> Self {
        Self(dur)
    }

    pub fn as_dur(&self) -> Duration {
        self.0
    }

    pub fn as_nanos(&self) -> u128 {
        self.0.as_nanos()
    }

    pub fn as_micros(&self) -> u128 {
        self.0.as_micros()
    }

    pub fn as_millis(&self) -> u128 {
        self.0.as_millis()
    }

    pub fn as_secs(&self) -> u64 {
        self.0.as_secs()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimestampedRawAudioBuffer {
    raw: RawAudioBuffer,

    start: Timestamp,
    stop: Timestamp,
}

impl TimestampedRawAudioBuffer {
    pub fn new(raw: RawAudioBuffer, start: Timestamp, stop: Timestamp) -> Self {
        Self { raw, start, stop }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.raw.as_slice()
    }
}

impl Message for TimestampedRawAudioBuffer {}
