use crate::util::vec_truncate_front;

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

    pub fn has_buffers(&self) -> bool {
        !self.buffers.is_empty()
    }

    pub fn has_bytes(&self) -> bool {
        self.buffers.front().is_some_and(|buf| buf.len() != 0)
    }

    pub fn push_buffer(&mut self, buffer: RawAudioBuffer) {
        self.buffers.push_back(buffer);
    }

    pub fn pop_buffer(&mut self) -> Option<RawAudioBuffer> {
        let mut buf = self.buffers.pop_front();

        let offset = self.front_buffer_offset;
        self.front_buffer_offset = 0;

        if let Some(buf) = buf.as_mut() {
            vec_truncate_front(buf.as_vec_mut(), offset);
        }

        buf
    }

    pub fn pop_buffer_formatted(&mut self, format: RawAudioFormat) -> Option<RawAudioBuffer> {
        if self
            .front_buffer_format()
            .is_some_and(|buf_format| buf_format == format)
        {
            return self.pop_buffer();
        }

        None
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
        if self
            .front_buffer_format()
            .is_some_and(|buf_format| buf_format == format)
        {
            return self.pop_bytes(desired).map(|(bytes, _)| bytes);
        }

        None
    }
}
