use core::audio_system::{
    audio::{RawAudioBuffer, RawAudioFormat},
    queue::RawAudioQueue,
};

use std::{mem::MaybeUninit, ptr};

pub struct CRawAudioQueueRC {
    ref_count: usize,
    queue: RawAudioQueue,
}

impl CRawAudioQueueRC {
    pub fn new(queue: RawAudioQueue) -> *mut Self {
        Box::into_raw(Box::new(Self {
            ref_count: 1,
            queue,
        }))
    }

    pub unsafe fn push_buffer(this: *mut Self, buffer: RawAudioBuffer) {
        (*this).queue.push_buffer(buffer);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_ref(
    queue: *mut CRawAudioQueueRC,
) -> *mut CRawAudioQueueRC {
    if queue.is_null() {
        return ptr::null_mut();
    }
    (*queue).ref_count += 1;

    queue
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_unref(queue: *mut CRawAudioQueueRC) {
    if queue.is_null() {
        return;
    }

    (*queue).ref_count -= 1;
    if (*queue).ref_count == 0 {
        drop(Box::from_raw(queue));
    }
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_front_buffer_format(
    queue: *mut CRawAudioQueueRC,
    format: *mut RawAudioFormat,
) -> libc::c_int {
    if queue.is_null() || format.is_null() {
        return 0;
    }

    if let Some(front_buffer_format) = (*queue).queue.front_buffer_format() {
        format.write(front_buffer_format);

        return 1;
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_read_bytes(
    queue: *mut CRawAudioQueueRC,
    bytes: *mut u8,
    nbytes: *mut libc::size_t,
    format: *mut RawAudioFormat,
) {
    if queue.is_null() || bytes.is_null() || nbytes.is_null() {
        return;
    }

    let Some((audio, audio_format)) = (*queue).queue.pop_bytes(*nbytes) else {
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
    queue: *mut CRawAudioQueueRC,
    bytes: *mut u8,
    nbytes: *mut libc::size_t,
    format: RawAudioFormat,
) {
    if queue.is_null() || bytes.is_null() || nbytes.is_null() {
        return;
    }

    let mut front_buffer_format = MaybeUninit::uninit();
    if ffone_raw_audio_queue_front_buffer_format(queue, front_buffer_format.as_mut_ptr()) != 0 {
        if front_buffer_format.assume_init() == format {
            ffone_raw_audio_queue_read_bytes(queue, bytes, nbytes, ptr::null_mut());

            return;
        }
    }

    nbytes.write(0);
}

pub struct RawAudioQueueRC(*mut CRawAudioQueueRC);

impl RawAudioQueueRC {
    pub fn new(queue: RawAudioQueue) -> Self {
        Self(CRawAudioQueueRC::new(queue))
    }

    pub fn push_buffer(&self, buffer: RawAudioBuffer) {
        unsafe {
            CRawAudioQueueRC::push_buffer(self.0, buffer);
        }
    }

    pub fn read_bytes(&self, bytes: &mut [u8]) -> Option<(usize, RawAudioFormat)> {
        let mut nbytes = bytes.len();
        let mut format = RawAudioFormat::default();

        unsafe {
            ffone_raw_audio_queue_read_bytes(
                self.0,
                bytes.as_mut_ptr(),
                &mut nbytes as *mut _,
                &mut format as *mut _,
            );
        }

        (nbytes > 0).then_some((nbytes, format))
    }

    pub fn read_bytes_formatted(&self, bytes: &mut [u8], format: RawAudioFormat) -> Option<usize> {
        let mut nbytes = bytes.len();

        unsafe {
            ffone_raw_audio_queue_read_bytes_formatted(
                self.0,
                bytes.as_mut_ptr(),
                &mut nbytes as *mut _,
                format,
            );
        }

        (nbytes > 0).then_some(nbytes)
    }
}

impl Clone for RawAudioQueueRC {
    fn clone(&self) -> Self {
        Self(unsafe { ffone_raw_audio_queue_ref(self.0) })
    }
}

impl Drop for RawAudioQueueRC {
    fn drop(&mut self) {
        unsafe {
            ffone_raw_audio_queue_unref(self.0);
        }
    }
}