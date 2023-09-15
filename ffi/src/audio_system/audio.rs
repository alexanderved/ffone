pub use core::audio_system::audio::RawAudioBuffer;
use core::audio_system::audio::RawAudioFormat;

use std::ptr;

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_buffer_drop(buffer: *mut libc::c_void) {
    if buffer.is_null() {
        return;
    }

    drop(Box::from_raw(buffer.cast::<RawAudioBuffer>()));
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_buffer_as_ptr(buffer: *const RawAudioBuffer) -> *const u8 {
    if buffer.is_null() {
        return ptr::null();
    }

    (*buffer).as_slice().as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_buffer_as_ptr_mut(buffer: *mut RawAudioBuffer) -> *mut u8 {
    if buffer.is_null() {
        return ptr::null_mut();
    }

    (*buffer).as_slice_mut().as_mut_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_buffer_format(
    buffer: *const RawAudioBuffer,
    format: *mut RawAudioFormat,
) -> libc::c_int {
    if buffer.is_null() {
        return 0;
    }
    *format = (*buffer).format();

    1
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_buffer_len(buffer: *const RawAudioBuffer) -> libc::size_t {
    if buffer.is_null() {
        return 0;
    }

    (*buffer).len() as libc::size_t
}

#[no_mangle]
pub unsafe extern "C" fn ffone_raw_audio_buffer_no_samples(
    buffer: *const RawAudioBuffer,
) -> libc::size_t {
    if buffer.is_null() {
        return 0;
    }

    (*buffer).no_samples() as libc::size_t
}
