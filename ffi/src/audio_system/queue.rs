use core::audio_system::{
    audio::{RawAudioBuffer, RawAudioFormat},
    queue::RawAudioQueue,
};

use std::{
    mem::{self, ManuallyDrop},
    ptr,
};

use crate::rc::{ffone_rc_alloc0, ffone_rc_ref, ffone_rc_unref};

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
    if queue.is_null() {
        return false;
    }

    (*queue).has_bytes()
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_has_buffers(queue: *mut RawAudioQueue) -> bool {
    if queue.is_null() {
        return false;
    }

    (*queue).has_buffers()
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_front_buffer_format(
    queue: *mut RawAudioQueue,
    format: *mut RawAudioFormat,
) -> bool {
    if queue.is_null() || format.is_null() {
        return false;
    }

    if let Some(front_buffer_format) = (*queue).front_buffer_format() {
        format.write(front_buffer_format);

        return true;
    }

    false
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_pop_buffer(
    queue: *mut RawAudioQueue,
) -> *mut RawAudioBuffer {
    if queue.is_null() {
        return ptr::null_mut();
    }

    let Some(buffer) = (*queue).pop_buffer() else {
        return ptr::null_mut();
    };

    Box::into_raw(Box::new(buffer))
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_pop_buffer_formatted(
    queue: *mut RawAudioQueue,
    format: RawAudioFormat,
    have_same_format: *mut bool,
) -> *mut RawAudioBuffer {
    if queue.is_null() {
        return ptr::null_mut();
    }

    if !have_same_format.is_null() {
        have_same_format.write(true);
    }

    let Some(front_buffer_format) = (*queue).front_buffer_format() else {
        return ptr::null_mut();
    };
    if front_buffer_format != format {
        if !have_same_format.is_null() {
            have_same_format.write(false);
        }

        return ptr::null_mut();
    }

    ffone_raw_audio_queue_pop_buffer(queue)
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_read_bytes(
    queue: *mut RawAudioQueue,
    bytes: *mut u8,
    nbytes: *mut libc::size_t,
    format: *mut RawAudioFormat,
) {
    if queue.is_null() || bytes.is_null() || nbytes.is_null() {
        return;
    }

    let Some((audio, audio_format)) = (*queue).pop_bytes(*nbytes) else {
        nbytes.write(0);
        return;
    };

    nbytes.write(audio.len().min(*nbytes));
    if !format.is_null() {
        format.write(audio_format);
    }

    ptr::copy_nonoverlapping(audio.as_ptr(), bytes, *nbytes);
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_read_bytes_formatted(
    queue: *mut RawAudioQueue,
    bytes: *mut u8,
    nbytes: *mut libc::size_t,
    format: RawAudioFormat,
    have_same_format: *mut bool,
) {
    if queue.is_null() || bytes.is_null() || nbytes.is_null() {
        return;
    }

    if !have_same_format.is_null() {
        have_same_format.write(true);
    }

    let Some(front_buffer_format) = (*queue).front_buffer_format() else {
        nbytes.write(0);

        return;
    };
    if front_buffer_format != format {
        nbytes.write(0);
        if !have_same_format.is_null() {
            have_same_format.write(false);
        }

        return;
    }

    ffone_raw_audio_queue_read_bytes(queue, bytes, nbytes, ptr::null_mut());
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

    pub fn read_bytes(&self, bytes: &mut [u8]) -> (usize, Option<RawAudioFormat>) {
        let popped_bytes = unsafe { (*self.0).pop_bytes(bytes.len()) };

        if let Some((available_bytes, format)) = popped_bytes {
            let available_nbytes = available_bytes.len();
            bytes[..available_nbytes].clone_from_slice(&available_bytes);

            return (available_nbytes, Some(format));
        }

        (0, None)
    }

    pub fn read_bytes_formatted(&self, bytes: &mut [u8], format: RawAudioFormat) -> (usize, bool) {
        let buffer_format = unsafe { (*self.0).front_buffer_format() };

        if buffer_format.is_some_and(|buffer_format| buffer_format == format) {
            let (nbytes, _) = self.read_bytes(bytes);

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
