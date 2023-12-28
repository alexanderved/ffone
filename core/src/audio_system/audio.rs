use mueue::*;

use crate::{
    error,
    util::{vec_truncate_front, ClockTime},
};

const NO_AUDIO_HEADER_BYTES: usize = 5;
const NO_CLOCK_TIME_BYTES: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuxedAudioBuffer(pub Vec<u8>);

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum AudioCodec {
    #[default]
    Unspecified,
    Opus,
}

impl TryFrom<u8> for AudioCodec {
    type Error = error::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let var = match value {
            0 => Self::Unspecified,
            1 => Self::Opus,
            _ => return Err(error::Error::IntToEnumCastFailed),
        };
        debug_assert_eq!(var as u8, value);

        Ok(var)
    }
}

impl TryFrom<&u8> for AudioCodec {
    type Error = error::Error;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        Self::try_from(*value)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EncodedAudioHeader {
    pub codec: AudioCodec,
    pub sample_rate: u32,
}

impl TryFrom<&[u8]> for EncodedAudioHeader {
    type Error = error::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let codec: AudioCodec = value
            .get(0)
            .ok_or(error::Error::EncodedAudioHeaderParseFailed)?
            .try_into()?;

        let sample_rate_bytes = value
            .get(1..NO_AUDIO_HEADER_BYTES)
            .ok_or(error::Error::EncodedAudioHeaderParseFailed)?
            .try_into()
            .expect("Failed to parse slice");
        let sample_rate = u32::from_be_bytes(sample_rate_bytes);

        Ok(Self { sample_rate, codec })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedAudioBuffer {
    pub header: EncodedAudioHeader,
    pub start_ts: Option<ClockTime>,
    pub data: Vec<u8>,
}

impl TryFrom<MuxedAudioBuffer> for EncodedAudioBuffer {
    type Error = error::Error;

    fn try_from(mut buf: MuxedAudioBuffer) -> Result<Self, Self::Error> {
        let header: EncodedAudioHeader = buf.0.as_slice().try_into()?;

        let start_ts_bytes = buf
            .0
            .get(NO_AUDIO_HEADER_BYTES..NO_AUDIO_HEADER_BYTES + NO_CLOCK_TIME_BYTES)
            .ok_or(error::Error::EncodedAudioHeaderParseFailed)?
            .try_into()
            .expect("Failed to parse slice");
        let start_ts_nanos = u64::from_be_bytes(start_ts_bytes);
        let start_ts = ClockTime::from_nanos(start_ts_nanos);

        vec_truncate_front(&mut buf.0, NO_AUDIO_HEADER_BYTES + NO_CLOCK_TIME_BYTES);
        let data = buf.0;

        Ok(Self {
            header,
            start_ts: Some(start_ts),
            data,
        })
    }
}

impl Message for EncodedAudioBuffer {}

#[repr(i8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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

    #[default]
    Unspecified,
}

impl RawAudioFormat {
    pub const fn no_bytes(self) -> usize {
        use RawAudioFormat::*;

        match self {
            U8 => 1,
            S16LE | S16BE => 2,
            S24LE | S24BE => 3,
            S32LE | S32BE | F32LE | F32BE => 4,
            Unspecified => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawAudioBuffer {
    data: Vec<u8>,
    format: RawAudioFormat,
    sample_rate: u32,
}

impl RawAudioBuffer {
    pub const fn new(data: Vec<u8>, format: RawAudioFormat, sample_rate: u32) -> Self {
        Self {
            data,
            format,
            sample_rate,
        }
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

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn no_samples(&self) -> usize {
        self.as_slice().len() / self.format().no_bytes()
    }

    pub fn duration(&self) -> ClockTime {
        let duration_nanos =
            self.no_samples() as u64 * ClockTime::NANOS_IN_SEC / self.sample_rate() as u64;

        ClockTime::from_nanos(duration_nanos)
    }

    pub fn truncate_front(&mut self, no_samples: usize) {
        let no_bytes = self.format().no_bytes();

        vec_truncate_front(self.as_vec_mut(), no_samples as usize * no_bytes);
    }

    pub fn truncate_duration_front(&mut self, cut_dur: ClockTime) {
        let sample_rate = self.sample_rate();
        let format = self.format();

        vec_truncate_front(self.as_vec_mut(), cut_dur.to_no_bytes(sample_rate, format));
    }

    pub fn truncate(&mut self, no_samples: usize) {
        let no_bytes = self.format().no_bytes();

        self.data.truncate(no_samples as usize * no_bytes);
    }

    pub fn truncate_duration(&mut self, cut_dur: ClockTime) {
        let sample_rate = self.sample_rate();
        let format = self.format();

        self.data.truncate(cut_dur.to_no_bytes(sample_rate, format));
    }

    pub fn split_at_timestamp(mut self, ts: ClockTime) -> (Self, Self) {
        let bytes = ts.to_no_bytes(self.sample_rate(), self.format());

        let new_buf = Self::new(self.data[bytes..].to_vec(), self.format, self.sample_rate);
        self.data.truncate(bytes);

        (self, new_buf)
    }
}

impl Message for RawAudioBuffer {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimestampedRawAudioBuffer {
    raw: RawAudioBuffer,

    start: Option<ClockTime>,
}

impl TimestampedRawAudioBuffer {
    pub const NULL: Self = Self::null();

    pub const fn new(raw: RawAudioBuffer, start: Option<ClockTime>) -> Self {
        Self { raw, start }
    }

    pub const fn null() -> Self {
        Self {
            raw: RawAudioBuffer::new(Vec::new(), RawAudioFormat::Unspecified, 0),
            start: None,
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

    pub fn format(&self) -> RawAudioFormat {
        self.raw.format()
    }

    pub fn sample_rate(&self) -> u32 {
        self.raw.sample_rate()
    }

    pub fn start(&self) -> Option<ClockTime> {
        self.start
    }

    pub fn stop(&self) -> Option<ClockTime> {
        self.start.map(|start| start + self.duration())
    }

    pub fn duration(&self) -> ClockTime {
        self.raw.duration()
    }

    pub fn truncate_front(&mut self, no_samples: usize) {
        let cut_dur = ClockTime::from_no_samples(no_samples, self.sample_rate());
        if let Some(start) = self.start.as_mut() {
            *start += cut_dur;
        }

        self.raw.truncate_front(no_samples);
    }

    pub fn truncate_duration_front(&mut self, cut_dur: ClockTime) {
        if let Some(start) = self.start.as_mut() {
            *start += cut_dur;
        }

        self.raw.truncate_duration_front(cut_dur);
    }

    pub fn truncate_duration(&mut self, cut_dur: ClockTime) {
        self.raw.truncate_duration(cut_dur);
    }

    pub fn split_at_timestamp(self, ts: ClockTime) -> (Self, Self) {
        let start = self.start;

        let (first_raw_buf, second_raw_buf) = self.into_raw().split_at_timestamp(ts);
        let first_ts_buf = Self::new(first_raw_buf, start);
        let second_ts_buf = Self::new(second_raw_buf, start.map(|start| start + ts));

        (first_ts_buf, second_ts_buf)
    }
}

impl Message for TimestampedRawAudioBuffer {}

#[derive(Debug, Clone, PartialEq, Eq)]
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
