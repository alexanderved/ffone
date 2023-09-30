use super::*;

use crate::audio_system::audio::{RawAudioBuffer, RawAudioFormat};
use crate::util::{ClockInfo, ClockTime};

use std::cell::Cell;
use std::sync::Arc;

use mueue::*;

const RAW_AUDIO_FORMAT: RawAudioFormat = RawAudioFormat::U8;
const SAMPLE_RATE: u32 = 8000;

struct FakeSystemClock(Cell<ClockTime>);

impl FakeSystemClock {
    fn new() -> Self {
        Self(Cell::new(ClockTime::ZERO))
    }

    fn move_forward(&self, time: ClockTime) {
        self.0.set(self.0.get() + time);
    }
}

impl Clock for FakeSystemClock {
    fn info(&self) -> ClockInfo {
        ClockInfo {
            name: String::from("Fake System Clock"),
        }
    }

    fn get_time(&self) -> ClockTime {
        self.0.get()
    }
}

#[test]
fn test_buffer_early_arrival() {
    let (send, _) = unidirectional_queue();
    let sys_clock = Arc::new(FakeSystemClock::new());
    let mut sync = Synchronizer::new(send, sys_clock.clone());

    let in_send = sync.create_input();
    let out_recv = sync.create_output();

    let first_buf = TimestampedRawAudioBuffer::new(
        RawAudioBuffer::new(
            vec![42; SAMPLE_RATE as usize],
            RAW_AUDIO_FORMAT,
            SAMPLE_RATE,
        ),
        Some(ClockTime::ZERO),
    );
    let _ = in_send.send(first_buf);
    let _ = sync.update(None);
    assert!(out_recv.recv().is_some());

    let early_buf = TimestampedRawAudioBuffer::new(
        RawAudioBuffer::new(
            vec![42; SAMPLE_RATE as usize],
            RAW_AUDIO_FORMAT,
            SAMPLE_RATE,
        ),
        Some(ClockTime::from_secs(1)),
    );
    sys_clock.move_forward(ClockTime::from_millis(500));
    let _ = in_send.send(early_buf);
    let _ = sync.update(None);
    assert!(out_recv.recv().is_none());

    sys_clock.move_forward(ClockTime::from_millis(500));
    let _ = sync.update(None);
    assert!(out_recv.recv().is_some());
}

#[test]
fn test_buffer_arrival_in_time() {
    let (send, _) = unidirectional_queue();
    let sys_clock = Arc::new(FakeSystemClock::new());
    let mut sync = Synchronizer::new(send, sys_clock.clone());

    let in_send = sync.create_input();
    let out_recv = sync.create_output();

    let first_buf = TimestampedRawAudioBuffer::new(
        RawAudioBuffer::new(
            vec![42; SAMPLE_RATE as usize],
            RAW_AUDIO_FORMAT,
            SAMPLE_RATE,
        ),
        Some(ClockTime::ZERO),
    );
    let _ = in_send.send(first_buf);
    let _ = sync.update(None);
    assert!(out_recv.recv().is_some());

    let in_time_buf = TimestampedRawAudioBuffer::new(
        RawAudioBuffer::new(
            vec![42; SAMPLE_RATE as usize],
            RAW_AUDIO_FORMAT,
            SAMPLE_RATE,
        ),
        Some(ClockTime::from_secs(1)),
    );
    sys_clock.move_forward(ClockTime::from_secs(1));
    let _ = in_send.send(in_time_buf);
    let _ = sync.update(None);
    assert!(out_recv.recv().is_some());
}

#[test]
fn test_buffer_late_arrival() {
    const DELAY: ClockTime = ClockTime::from_millis(250);

    let (send, _) = unidirectional_queue();
    let sys_clock = Arc::new(FakeSystemClock::new());
    let mut sync = Synchronizer::new(send, sys_clock.clone());

    let in_send = sync.create_input();
    let out_recv = sync.create_output();

    let reference_buffer = ResizableRawAudioBuffer::new(
        RawAudioBuffer::new(
            vec![42; SAMPLE_RATE as usize],
            RAW_AUDIO_FORMAT,
            SAMPLE_RATE,
        ),
        SAMPLE_RATE as usize - 250 * SAMPLE_RATE as usize / ClockTime::MILLIS_IN_SEC as usize,
    );

    let first_buf = TimestampedRawAudioBuffer::new(
        RawAudioBuffer::new(
            vec![42; SAMPLE_RATE as usize],
            RAW_AUDIO_FORMAT,
            SAMPLE_RATE,
        ),
        Some(ClockTime::ZERO),
    );
    let _ = in_send.send(first_buf);
    let _ = sync.update(None);
    assert!(out_recv.recv().is_some());

    let late_buf = TimestampedRawAudioBuffer::new(
        RawAudioBuffer::new(
            vec![42; SAMPLE_RATE as usize],
            RAW_AUDIO_FORMAT,
            SAMPLE_RATE,
        ),
        Some(ClockTime::from_secs(1)),
    );
    sys_clock.move_forward(ClockTime::from_secs(1) + DELAY);
    let _ = in_send.send(late_buf);
    let _ = sync.update(None);
    assert_eq!(out_recv.recv(), Some(reference_buffer));
}

#[test]
fn test_non_monotonous_timestamp() {
    let (send, _) = unidirectional_queue();
    let sys_clock = Arc::new(FakeSystemClock::new());
    let mut sync = Synchronizer::new(send, sys_clock.clone());

    let in_send = sync.create_input();
    let out_recv = sync.create_output();

    let reference_buffer = ResizableRawAudioBuffer::new(
        RawAudioBuffer::new(
            vec![42; SAMPLE_RATE as usize / 2],
            RAW_AUDIO_FORMAT,
            SAMPLE_RATE,
        ),
        SAMPLE_RATE as usize - 500 * SAMPLE_RATE as usize / ClockTime::MILLIS_IN_SEC as usize,
    );

    let first_buf = TimestampedRawAudioBuffer::new(
        RawAudioBuffer::new(
            vec![42; SAMPLE_RATE as usize],
            RAW_AUDIO_FORMAT,
            SAMPLE_RATE,
        ),
        Some(ClockTime::ZERO),
    );
    let _ = in_send.send(first_buf);
    let _ = sync.update(None);
    assert!(out_recv.recv().is_some());

    let non_mono_buf = TimestampedRawAudioBuffer::new(
        RawAudioBuffer::new(
            vec![42; SAMPLE_RATE as usize],
            RAW_AUDIO_FORMAT,
            SAMPLE_RATE,
        ),
        Some(ClockTime::from_millis(500)),
    );
    sys_clock.move_forward(ClockTime::from_millis(750));
    let _ = in_send.send(non_mono_buf);
    let _ = sync.update(None);
    assert!(out_recv.recv().is_none());

    sys_clock.move_forward(ClockTime::from_millis(250));
    let _ = sync.update(None);
    assert_eq!(out_recv.recv(), Some(reference_buffer));
}
