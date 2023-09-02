pub mod audio_decoder;
pub mod resizer;
pub mod sync;
pub mod virtual_microphone;

use audio_decoder::*;
use resizer::AudioResizer;
use sync::*;
use virtual_microphone::*;

use super::element::{AsAudioSink, AsAudioSource};

use crate::error;
use crate::util::{ControlFlow, Runnable, RunnableStateMachine};

macro_rules! add_pipeline_element {
    (
        @element $elem:ty;
        @long_name $func:ident;
        @name $name:ident;
        $(
            @prev $prev:ident;
            $( @prev_on_set $prev_on_set:block; )?
            $( @prev_on_take $prev_on_take:block; )?
        )?
        $(
            @next $next:ident;
            $( @next_on_set $next_on_set:block; )?
            $( @next_on_take $next_on_take:block; )?
        )?
    ) => {
        paste::paste! {
            pub(super) fn [< set_ $func >](&mut self, mut elem: $elem) {
                if self.is_running {
                    elem.on_start();
                }

                $(
                    if let Some($prev) = self.$prev.as_mut() {
                        $prev.as_audio_source_mut().chain(elem.as_audio_sink_mut());
                        $( $prev_on_set; )?
                    }
                )?

                $(
                    if let Some($next) = self.$next.as_mut() {
                        elem.as_audio_source_mut().chain($next.as_audio_sink_mut());
                        $( $next_on_set; )?
                    }
                )?

                self.$name = Some(elem);
            }

            pub(super) fn [< take_ $func >](&mut self) -> Option<$elem> {
                $(
                    if let Some($prev) = self.$prev.as_mut() {
                        $prev.as_audio_source_mut().unset_output();
                        $( $prev_on_take; )?
                    }
                )?

                $(
                    if let Some($next) = self.$next.as_mut() {
                        $next.as_audio_sink_mut().unset_input();
                        $( $next_on_take; )?
                    }
                )?

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

            pub(super) fn $func(&self) -> Option<&$elem> {
                self.$name.as_ref()
            }

            pub(super) fn [< $func _mut >](&mut self) -> Option<&mut $elem> {
                self.$name.as_mut()
            }
        }
    };
}

pub(super) type AudioPipelineStateMachine = RunnableStateMachine<AudioPipeline>;

// TODO: Make extendable and polymorphic.
pub(super) struct AudioPipeline {
    dec: Option<Box<dyn AudioDecoder>>,
    sync: Option<Synchronizer>,
    resizer: Option<AudioResizer>,
    mic: Option<Box<dyn VirtualMicrophone>>,

    is_running: bool,
}

#[allow(dead_code)]
impl AudioPipeline {
    pub(super) fn new() -> Self {
        Self {
            dec: None,
            sync: None,
            resizer: None,
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
        @next resizer;
    }

    add_pipeline_element! {
        @element AudioResizer;
        @long_name resizer;
        @name resizer;
        @prev sync;
        @next mic;
    }

    add_pipeline_element! {
        @element Box<dyn VirtualMicrophone>;
        @long_name virtual_microphone;
        @name mic;
        @prev resizer;
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
