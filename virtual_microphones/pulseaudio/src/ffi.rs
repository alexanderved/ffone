use core::audio_system::queue::RawAudioQueue;
use std::marker::{PhantomData, PhantomPinned};

#[repr(C)]
pub struct FFonePAContext {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

#[repr(C)]
pub struct FFonePAStream {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

extern "C" {
    #[allow(improper_ctypes)]
    pub(crate) fn ffone_pa_ctx_new(queue: *mut RawAudioQueue) -> *mut FFonePAContext;
    pub(crate) fn ffone_pa_ctx_get_stream(pa_ctx: *mut FFonePAContext) -> *mut FFonePAStream;
    pub(crate) fn ffone_pa_ctx_update(
        pa_ctx: *mut FFonePAContext,
        block: libc::c_int,
    ) -> libc::c_int;

    pub(crate) fn ffone_pa_stream_get_time(stream: *mut FFonePAStream) -> u64;
}
