use crate::error;
use crate::util::{ControlFlow, Runnable, RunnableStateMachine};

use super::audio_decoder::*;
use super::element::{AsAudioFilter, AsAudioSink};
use super::sync::*;
use super::virtual_microphone::*;

pub(super) type AudioPipelineStateMachine = RunnableStateMachine<AudioPipeline>;

// TODO: Make extendable and polymorphic.
pub(super) struct AudioPipeline {
    dec: Option<Box<dyn AudioDecoder>>,
    sync: Option<Synchronizer>,
    mic: Option<Box<dyn VirtualMicrophone>>,

    is_running: bool,
}

#[allow(dead_code)]
impl AudioPipeline {
    pub(super) fn new() -> Self {
        Self {
            dec: None,
            sync: None,
            mic: None,

            is_running: false,
        }
    }

    pub(super) fn set_audio_decoder(&mut self, mut dec: Box<dyn AudioDecoder>) {
        if self.is_running {
            dec.on_start();
        }

        if let Some(sync) = self.sync.as_mut() {
            dec.chain(sync.as_audio_sink_mut());
        }

        self.dec = Some(dec);
    }

    pub(super) fn take_audio_decoder(&mut self) -> Option<Box<dyn AudioDecoder>> {
        self.dec.take().map(|mut dec| {
            if self.is_running {
                dec.on_stop();
            }
            dec
        })
    }

    pub(super) fn replace_audio_decoder(
        &mut self,
        dec: Box<dyn AudioDecoder>,
    ) -> Option<Box<dyn AudioDecoder>> {
        let old_dec = self.take_audio_decoder();
        self.set_audio_decoder(dec);

        old_dec
    }

    pub(super) fn set_synchronizer(&mut self, mut sync: Synchronizer) {
        if self.is_running {
            sync.on_start();
        }

        if let Some(dec) = self.dec.as_mut() {
            dec.chain(sync.as_audio_sink_mut());
        }

        if let Some(mic) = self.mic.as_mut() {
            sync.as_audio_filter_mut().chain(mic.as_audio_sink_mut());
        }

        self.sync = Some(sync);
    }

    pub(super) fn take_synchronizer(&mut self) -> Option<Synchronizer> {
        self.sync.take().map(|mut sync| {
            if self.is_running {
                sync.on_stop();
            }
            sync
        })
    }

    pub(super) fn replace_synchronizer(&mut self, sync: Synchronizer) -> Option<Synchronizer> {
        let old_sync = self.take_synchronizer();
        self.set_synchronizer(sync);

        old_sync
    }

    pub(super) fn set_virtual_microphone(&mut self, mut mic: Box<dyn VirtualMicrophone>) {
        if self.is_running {
            mic.on_start();
        }

        if let Some(sync) = self.sync.as_mut() {
            sync.as_audio_filter_mut().chain(mic.as_audio_sink_mut());
        }

        self.mic = Some(mic);
    }

    pub(super) fn take_virtual_microphone(&mut self) -> Option<Box<dyn VirtualMicrophone>> {
        self.mic.take().map(|mut mic| {
            if self.is_running {
                mic.on_stop();
            }
            mic
        })
    }

    pub(super) fn replace_virtual_microphone(
        &mut self,
        mic: Box<dyn VirtualMicrophone>,
    ) -> Option<Box<dyn VirtualMicrophone>> {
        let old_mic = self.take_virtual_microphone();
        self.set_virtual_microphone(mic);

        old_mic
    }
}

impl Runnable for AudioPipeline {
    fn update(&mut self, flow: &mut ControlFlow) -> error::Result<()> {
        self.dec.as_mut().map(|dec| dec.update(&mut *flow));
        self.sync.as_mut().map(|sync| sync.update(&mut *flow));
        self.mic.as_mut().map(|mic| mic.update(&mut *flow));

        Ok(())
    }

    fn on_start(&mut self) {
        self.is_running = true;

        self.dec.as_mut().map(Runnable::on_start);
        self.sync.as_mut().map(Runnable::on_start);
        self.mic.as_mut().map(Runnable::on_start);
    }

    fn on_stop(&mut self) {
        self.is_running = false;

        self.dec.as_mut().map(Runnable::on_stop);
        self.sync.as_mut().map(Runnable::on_stop);
        self.mic.as_mut().map(Runnable::on_stop);
    }
}
