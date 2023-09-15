use super::audio::{RawAudioBuffer, RawAudioFormat};

use std::collections::VecDeque;

pub struct RawAudioQueue {
    buffers: VecDeque<RawAudioBuffer>,
    front_buffer_offset: usize,
}

impl RawAudioQueue {
    pub fn new() -> Self {
        Self {
            buffers: VecDeque::new(),
            front_buffer_offset: 0,
        }
    }

    pub fn front_buffer_format(&self) -> Option<RawAudioFormat> {
        self.buffers.front().map(RawAudioBuffer::format)
    }

    pub fn front_buffer_sample_rate(&self) -> Option<u32> {
        self.buffers.front().map(RawAudioBuffer::sample_rate)
    }

    pub fn has_buffers(&self) -> bool {
        !self.buffers.is_empty()
    }

    pub fn has_bytes(&self) -> bool {
        self.buffers.front().is_some_and(|buf| buf.len() != 0)
    }

    pub fn push_buffer(&mut self, buffer: RawAudioBuffer) {
        self.buffers.push_back(buffer);
    }

    pub fn pop_bytes(&mut self, desired: usize) -> Option<(Vec<u8>, RawAudioFormat, u32)> {
        let res = self.buffers.front().map(|front_buffer| {
            let available = front_buffer.len() - self.front_buffer_offset;

            let start = self.front_buffer_offset;
            let end = self.front_buffer_offset + desired.min(available);
            self.front_buffer_offset = end;

            (
                front_buffer.as_slice()[start..end].to_vec(),
                front_buffer.format(),
                front_buffer.sample_rate(),
            )
        });

        if self
            .buffers
            .front()
            .is_some_and(|buf| self.front_buffer_offset >= buf.len())
        {
            let front_buffer = self.buffers.pop_front().unwrap();
            self.front_buffer_offset -= front_buffer.len();
        }

        res
    }

    pub fn pop_bytes_with_props(
        &mut self,
        desired: usize,
        format: RawAudioFormat,
        sample_rate: u32,
    ) -> Option<Vec<u8>> {
        let Some(front_buffer_format) = self.front_buffer_format() else {
            return None;
        };
        let Some(front_buffer_sample_rate) = self.front_buffer_sample_rate() else {
            return None;
        };

        if front_buffer_format == format && front_buffer_sample_rate == sample_rate {
            return self.pop_bytes(desired).map(|(bytes, _, _)| bytes);
        }

        None
    }
}
