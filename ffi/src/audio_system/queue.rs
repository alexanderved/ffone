use core::audio_system::{
    audio::{RawAudioBuffer, RawAudioFormat},
    queue::RawAudioQueue,
};

use std::{
    mem::{self, ManuallyDrop},
    ptr,
};

use crate::rc::{ffone_rc_alloc0, ffone_rc_is_destructed, ffone_rc_ref, ffone_rc_unref};

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_new() -> *mut RawAudioQueue {
    let rc = ffone_rc_alloc0(
        mem::size_of::<RawAudioQueue>(),
        Some(ffone_raw_audio_queue_dtor),
    )
    .cast::<RawAudioQueue>();
    if rc.is_null() {
        return ptr::null_mut();
    }

    rc.write(RawAudioQueue::new());
    rc
}

#[no_mangle]
unsafe extern "C" fn ffone_raw_audio_queue_dtor(queue: *mut libc::c_void) {
    if queue.is_null() {
        return;
    }

    queue.cast::<RawAudioQueue>().drop_in_place();
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_has_bytes(queue: *mut RawAudioQueue) -> bool {
    if queue.is_null() || ffone_rc_is_destructed(queue.cast()) {
        return false;
    }

    (*queue).has_bytes()
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_has_buffers(queue: *mut RawAudioQueue) -> bool {
    if queue.is_null() || ffone_rc_is_destructed(queue.cast()) {
        return false;
    }

    (*queue).has_buffers()
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_front_buffer_format(
    queue: *mut RawAudioQueue,
    format: *mut RawAudioFormat,
) -> bool {
    if queue.is_null() || ffone_rc_is_destructed(queue.cast()) || format.is_null() {
        return false;
    }

    if let Some(front_buffer_format) = (*queue).front_buffer_format() {
        format.write(front_buffer_format);

        return true;
    }

    false
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_front_buffer_sample_rate(
    queue: *mut RawAudioQueue,
    sample_rate: *mut u32,
) -> bool {
    if queue.is_null() || ffone_rc_is_destructed(queue.cast()) || sample_rate.is_null() {
        return false;
    }

    if let Some(front_buffer_sample_rate) = (*queue).front_buffer_sample_rate() {
        sample_rate.write(front_buffer_sample_rate);

        return true;
    }

    false
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_read_bytes(
    queue: *mut RawAudioQueue,
    bytes: *mut u8,
    nbytes: *mut libc::size_t,
    format: *mut RawAudioFormat,
    sample_rate: *mut u32,
) {
    if queue.is_null()
        || ffone_rc_is_destructed(queue.cast())
        || bytes.is_null()
        || nbytes.is_null()
    {
        return;
    }

    let Some((audio, audio_format, audio_sample_rate)) = (*queue).pop_bytes(*nbytes) else {
        nbytes.write(0);
        return;
    };

    nbytes.write(audio.len().min(*nbytes));
    if !format.is_null() {
        format.write(audio_format);
    }
    if !sample_rate.is_null() {
        sample_rate.write(audio_sample_rate);
    }

    ptr::copy_nonoverlapping(audio.as_ptr(), bytes, *nbytes);
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_read_bytes_with_props(
    queue: *mut RawAudioQueue,
    bytes: *mut u8,
    nbytes: *mut libc::size_t,
    format: RawAudioFormat,
    sample_rate: u32,
    have_same_props: *mut bool,
) {
    if queue.is_null()
        || ffone_rc_is_destructed(queue.cast())
        || bytes.is_null()
        || nbytes.is_null()
    {
        return;
    }

    if !have_same_props.is_null() {
        have_same_props.write(true);
    }

    let Some(front_buffer_format) = (*queue).front_buffer_format() else {
        nbytes.write(0);

        return;
    };
    if front_buffer_format != format {
        nbytes.write(0);
        if !have_same_props.is_null() {
            have_same_props.write(false);
        }

        return;
    }

    let Some(front_buffer_sample_rate) = (*queue).front_buffer_sample_rate() else {
        nbytes.write(0);

        return;
    };
    if front_buffer_sample_rate != sample_rate {
        nbytes.write(0);
        if !have_same_props.is_null() {
            have_same_props.write(false);
        }

        return;
    }

    ffone_raw_audio_queue_read_bytes(queue, bytes, nbytes, ptr::null_mut(), ptr::null_mut());
}

pub struct RawAudioQueueRC(*mut RawAudioQueue);

impl RawAudioQueueRC {
    pub fn new() -> Option<Self> {
        let queue = unsafe { ffone_raw_audio_queue_new() };

        if !queue.is_null() {
            Some(Self(queue))
        } else {
            None
        }
    }

    pub fn into_raw(self) -> *mut RawAudioQueue {
        ManuallyDrop::new(self).0
    }

    pub fn push_buffer(&self, buffer: RawAudioBuffer) {
        unsafe {
            (*self.0).push_buffer(buffer);
        }
    }

    pub fn read_bytes(&self, bytes: &mut [u8]) -> (usize, Option<RawAudioFormat>, Option<u32>) {
        let popped_bytes = unsafe { (*self.0).pop_bytes(bytes.len()) };

        if let Some((available_bytes, format, sample_rate)) = popped_bytes {
            let available_nbytes = available_bytes.len();
            bytes[..available_nbytes].clone_from_slice(&available_bytes);

            return (available_nbytes, Some(format), Some(sample_rate));
        }

        (0, None, None)
    }

    pub fn read_bytes_with_props(
        &self,
        bytes: &mut [u8],
        format: RawAudioFormat,
        sample_rate: u32,
    ) -> (usize, bool) {
        let Some(buffer_format) = unsafe { &*self.0 }.front_buffer_format() else {
            return (0, false);
        };
        let Some(buffer_sample_rate) = unsafe { &*self.0 }.front_buffer_sample_rate() else {
            return (0, false);
        };

        if buffer_format == format && buffer_sample_rate == sample_rate {
            let (nbytes, _, _) = self.read_bytes(bytes);

            return (nbytes, true);
        }

        (0, false)
    }
}

impl Clone for RawAudioQueueRC {
    fn clone(&self) -> Self {
        Self(unsafe { ffone_rc_ref(self.0.cast()).cast() })
    }
}

impl Drop for RawAudioQueueRC {
    fn drop(&mut self) {
        unsafe {
            ffone_rc_unref(self.0.cast());
        }
    }
}
