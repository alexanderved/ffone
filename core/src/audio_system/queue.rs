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
        self.buffers.front().map(|buf| buf.format())
    }

    pub fn push_buffer(&mut self, buffer: RawAudioBuffer) {
        self.buffers.push_back(buffer);
    }

    pub fn pop_bytes(&mut self, desired: usize) -> Option<(Vec<u8>, RawAudioFormat)> {
        let res = self.buffers.front().map(|front_buffer| {
            let available = front_buffer.len() - self.front_buffer_offset;

            let start = self.front_buffer_offset;
            let end = self.front_buffer_offset + desired.min(available);
            self.front_buffer_offset = end;

            (
                front_buffer.as_slice()[start..end].to_vec(),
                front_buffer.format(),
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

    pub fn pop_bytes_formatted(
        &mut self,
        desired: usize,
        format: RawAudioFormat,
    ) -> Option<Vec<u8>> {
        if !self
            .front_buffer_format()
            .is_some_and(|buf_format| buf_format == format)
        {
            return None;
        }

        self.pop_bytes(desired).map(|(bytes, _)| bytes)
    }
}
