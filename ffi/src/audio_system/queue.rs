use core::{audio_system::{
    audio::{RawAudioBuffer, RawAudioFormat},
    queue::RawAudioQueue,
}, util::ClockTime};

use std::{
    mem::{self, ManuallyDrop},
    ptr,
};

use crate::rc::{ffone_rc_alloc0, ffone_rc_ref, ffone_rc_unref, ffone_rc_lock, ffone_rc_unlock};

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_new(max_duration: u64) -> *mut RawAudioQueue {
    let rc = ffone_rc_alloc0(
        mem::size_of::<RawAudioQueue>(),
        Some(ffone_raw_audio_queue_dtor),
    )
    .cast::<RawAudioQueue>();
    if rc.is_null() {
        return ptr::null_mut();
    }

    rc.write(RawAudioQueue::new(ClockTime::from_nanos(max_duration)));
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
pub unsafe extern "C" fn ffone_raw_audio_queue_has_bytes_locked(
    queue: *mut RawAudioQueue
) -> bool {
    if queue.is_null() {
        return false;
    }

    (*queue).has_bytes()
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_has_bytes(queue: *mut RawAudioQueue) -> bool {
    if queue.is_null() {
        return false;
    }

    ffone_rc_lock(queue.cast());
    let has_bytes = (*queue).has_bytes();
    ffone_rc_unlock(queue.cast());

    has_bytes
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_has_buffers(queue: *mut RawAudioQueue) -> bool {
    if queue.is_null() {
        return false;
    }

    ffone_rc_lock(queue.cast());
    let has_buffers = (*queue).has_buffers();
    ffone_rc_unlock(queue.cast());

    has_buffers
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_front_buffer_format(
    queue: *mut RawAudioQueue,
    format: *mut RawAudioFormat,
) -> bool {
    if queue.is_null() || format.is_null() {
        return false;
    }

    ffone_rc_lock(queue.cast());

    if let Some(front_buffer_format) = (*queue).front_buffer_format() {
        format.write(front_buffer_format);
        ffone_rc_unlock(queue.cast());

        return true;
    }

    ffone_rc_unlock(queue.cast());

    false
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_front_buffer_sample_rate(
    queue: *mut RawAudioQueue,
    sample_rate: *mut u32,
) -> bool {
    if queue.is_null() || sample_rate.is_null() {
        return false;
    }

    ffone_rc_lock(queue.cast());

    if let Some(front_buffer_sample_rate) = (*queue).front_buffer_sample_rate() {
        sample_rate.write(front_buffer_sample_rate);
        ffone_rc_unlock(queue.cast());

        return true;
    }

    ffone_rc_unlock(queue.cast());

    false
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_read_bytes_locked(
    queue: *mut RawAudioQueue,
    bytes: *mut u8,
    nbytes: *mut libc::size_t,
    format: *mut RawAudioFormat,
    sample_rate: *mut u32,
) {
    if queue.is_null()
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
pub unsafe extern "C" fn ffone_raw_audio_queue_read_bytes(
    queue: *mut RawAudioQueue,
    bytes: *mut u8,
    nbytes: *mut libc::size_t,
    format: *mut RawAudioFormat,
    sample_rate: *mut u32,
) {
    if queue.is_null() {
        return;
    }

    ffone_rc_lock(queue.cast());
    ffone_raw_audio_queue_read_bytes_locked(queue, bytes, nbytes, format, sample_rate);
    ffone_rc_unlock(queue.cast());
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_queue_read_bytes_with_props_locked(
    queue: *mut RawAudioQueue,
    bytes: *mut u8,
    nbytes: *mut libc::size_t,
    format: RawAudioFormat,
    sample_rate: u32,
    have_same_props: *mut bool,
) {
    if queue.is_null()
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

    ffone_raw_audio_queue_read_bytes_locked(
        queue,
        bytes,
        nbytes,
        ptr::null_mut(),
        ptr::null_mut()
    );
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
    if queue.is_null() {
        return;
    }

    ffone_rc_lock(queue.cast());
    ffone_raw_audio_queue_read_bytes_with_props_locked(
        queue,
        bytes,
        nbytes,
        format,
        sample_rate,
        have_same_props,
    );
    ffone_rc_unlock(queue.cast());
}

pub struct RawAudioQueueRC(*mut RawAudioQueue);

impl RawAudioQueueRC {
    pub fn new(max_duration: ClockTime) -> Option<Self> {
        let queue = unsafe { ffone_raw_audio_queue_new(max_duration.as_nanos()) };

        if !queue.is_null() {
            Some(Self(queue))
        } else {
            None
        }
    }

    pub fn has_buffers(&self) -> bool {
        unsafe {
            ffone_raw_audio_queue_has_buffers(self.0)
        }
    }

    pub fn has_bytes(&self) -> bool {
        unsafe {
            ffone_raw_audio_queue_has_bytes(self.0)
        }
    }

    pub fn no_buffers(&self) -> usize {
        unsafe {
            ffone_rc_lock(self.0.cast());
            let no_buffers = (*self.0).no_buffers();
            ffone_rc_unlock(self.0.cast());

            no_buffers
        }
    }

    pub fn no_bytes(&self) -> usize {
        unsafe {
            ffone_rc_lock(self.0.cast());
            let no_bytes = (*self.0).no_bytes();
            ffone_rc_unlock(self.0.cast());

            no_bytes
        }
    }

    pub fn duration(&self) -> ClockTime {
        unsafe {
            ffone_rc_lock(self.0.cast());
            let duration = (*self.0).duration();
            ffone_rc_unlock(self.0.cast());

            duration
        }
    }

    pub fn available_duration(&self) -> ClockTime {
        unsafe {
            ffone_rc_lock(self.0.cast());
            let available_duration = (*self.0).available_duration();
            ffone_rc_unlock(self.0.cast());

            available_duration
        }
    }

    pub fn into_raw(self) -> *mut RawAudioQueue {
        ManuallyDrop::new(self).0
    }

    pub fn push_buffer(&self, buffer: RawAudioBuffer) {
        unsafe {
            ffone_rc_lock(self.0.cast());
            (*self.0).push_buffer(buffer);
            ffone_rc_unlock(self.0.cast());
        }
    }

    pub fn read_bytes(&self, bytes: &mut [u8]) -> (usize, Option<RawAudioFormat>, Option<u32>) {
        unsafe { ffone_rc_lock(self.0.cast()); }

        let popped_bytes = unsafe { (*self.0).pop_bytes(bytes.len()) };

        if let Some((available_bytes, format, sample_rate)) = popped_bytes {
            let available_nbytes = available_bytes.len();
            bytes[..available_nbytes].clone_from_slice(&available_bytes);

            unsafe { ffone_rc_unlock(self.0.cast()); }

            return (available_nbytes, Some(format), Some(sample_rate));
        }

        unsafe { ffone_rc_unlock(self.0.cast()); }

        (0, None, None)
    }

    pub fn read_bytes_with_props(
        &self,
        bytes: &mut [u8],
        format: RawAudioFormat,
        sample_rate: u32,
    ) -> (usize, bool) {
        unsafe { ffone_rc_lock(self.0.cast()); }

        let Some(buffer_format) = unsafe { &*self.0 }.front_buffer_format() else {
            unsafe { ffone_rc_unlock(self.0.cast()); }

            return (0, false);
        };
        let Some(buffer_sample_rate) = unsafe { &*self.0 }.front_buffer_sample_rate() else {
            unsafe { ffone_rc_unlock(self.0.cast()); }

            return (0, false);
        };

        if buffer_format == format && buffer_sample_rate == sample_rate {
            let (nbytes, _, _) = self.read_bytes(bytes);

            unsafe { ffone_rc_unlock(self.0.cast()); }

            return (nbytes, true);
        }

        unsafe { ffone_rc_unlock(self.0.cast()); }

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
