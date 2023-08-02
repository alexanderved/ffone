use crate::error;
use crate::util::{ControlFlow, Runnable, RunnableStateMachine};

use super::audio_decoder::*;
use super::downsampler::AudioDownsampler;
use super::element::{AsAudioSource, AsAudioSink};
use super::sync::*;
use super::virtual_microphone::*;

macro_rules! add_pipeline_element {
    (
        @element $elem:ty;
        @long_name $func:ident;
        @name $name:ident;
        $( @prev $prev:ident; )?
        $( @next $next:ident; )?
    ) => {
        paste::paste! {
            pub(super) fn [< set_ $func >](&mut self, mut elem: $elem) {
                if self.is_running {
                    elem.on_start();
                }

                $(
                    if let Some($prev) = self.$prev.as_mut() {
                        $prev.as_audio_source_mut().chain(elem.as_audio_sink_mut());
                    }
                )?

                $(
                    if let Some($next) = self.$next.as_mut() {
                        elem.as_audio_source_mut().chain($next.as_audio_sink_mut());
                    }
                )?
        
                self.$name = Some(elem);
            }

            pub(super) fn [< take_ $func >](&mut self) -> Option<$elem> {
                self.$name.take().map(|mut elem| {
                    if self.is_running {
                        elem.on_stop();
                    }
                    elem
                })
            }

            pub(super) fn [< has_ $func >](&self) -> bool {
                self.$name.is_some()
            }
        }
    };
}

pub(super) type AudioPipelineStateMachine = RunnableStateMachine<AudioPipeline>;

// TODO: Make extendable and polymorphic.
pub(super) struct AudioPipeline {
    dec: Option<Box<dyn AudioDecoder>>,
    sync: Option<Synchronizer>,
    downsampler: Option<AudioDownsampler>,
    mic: Option<Box<dyn VirtualMicrophone>>,

    is_running: bool,
}

#[allow(dead_code)]
impl AudioPipeline {
    pub(super) fn new() -> Self {
        Self {
            dec: None,
            sync: None,
            downsampler: None,
            mic: None,

            is_running: false,
        }
    }

    add_pipeline_element! {
        @element Box<dyn AudioDecoder>;
        @long_name audio_decoder;
        @name dec;
        @next sync;
    }

    add_pipeline_element! {
        @element Synchronizer;
        @long_name synchronizer;
        @name sync;
        @prev dec;
        @next downsampler;
    }

    add_pipeline_element! {
        @element AudioDownsampler;
        @long_name downsampler;
        @name downsampler;
        @prev sync;
        @next mic;
    }

    add_pipeline_element! {
        @element Box<dyn VirtualMicrophone>;
        @long_name virtual_microphone;
        @name mic;
        @prev downsampler;
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
        self.dec.as_mut().map(Runnable::on_start);
        self.sync.as_mut().map(Runnable::on_start);
        self.mic.as_mut().map(Runnable::on_start);

        self.is_running = true;
    }

    fn on_stop(&mut self) {
        self.is_running = false;

        self.dec.as_mut().map(Runnable::on_stop);
        self.sync.as_mut().map(Runnable::on_stop);
        self.mic.as_mut().map(Runnable::on_stop);
    }
}