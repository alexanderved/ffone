use core::util::{Clock, ClockInfo, ClockTime};

use ffi::rc::ffone_rc_unref;

use super::ffi::ffone_pa_stream_get_time;
use super::ffi::FFonePAStream;

pub struct PAClock(*mut FFonePAStream);

impl PAClock {
    pub unsafe fn new(stream: *mut FFonePAStream) -> Self {
        Self(stream)
    }
}

impl Clock for PAClock {
    fn info(&self) -> ClockInfo {
        ClockInfo {
            name: String::from("Pulseaudio Clock")
        }
    }

    fn get_time(&self) -> ClockTime {
        let usec = unsafe {
            ffone_pa_stream_get_time(self.0)
        };

        ClockTime::from_micros(usec)
    }
}

impl Drop for PAClock {
    fn drop(&mut self) {
        unsafe {
            ffone_rc_unref(self.0.cast())
        }
    }
}