use core::audio_system::queue::RawAudioQueue;
use std::marker::{PhantomData, PhantomPinned};

#[repr(C)]
pub struct FFonePACore {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

#[repr(C)]
pub struct FFonePAStream {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

extern "C" {
    pub(crate) fn ffone_pa_core_new() -> *mut FFonePACore;

    #[allow(improper_ctypes)]
    pub(crate) fn ffone_pa_stream_new(
        core: *mut FFonePACore,
        queue: *mut RawAudioQueue,
    ) -> *mut FFonePAStream;
    pub(crate) fn ffone_pa_stream_play(stream: *mut FFonePAStream);
    pub(crate) fn ffone_pa_stream_get_time(stream: *mut FFonePAStream) -> u64;
}
