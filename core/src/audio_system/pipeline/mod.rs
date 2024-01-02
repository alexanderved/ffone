pub mod audio_decoder;
pub mod demuxer;
pub mod resizer;
pub mod sync;
pub mod virtual_microphone;

use audio_decoder::*;
use demuxer::*;
use resizer::AudioResizer;
use sync::*;
use virtual_microphone::*;

use super::element::{AsAudioSink, AsAudioSource};

use crate::error;
use crate::util::{Runnable, RunnableStateMachine};

macro_rules! add_pipeline_element {
    (
        @element $elem:ty;

        @long_name $func:ident;
        @name $name:ident;

        $( @prev $prev:ident; )?
        $( @next $next:ident; )?

        $(
            @modify_on_set ($elem_on_set:ident: $elem_on_set_ty:ty) => $on_set_mod:block;
        )*

        $(
            @modify_on_take ($elem_on_take:ident: $elem_on_take_ty:ty) => $on_take_mod:block;
        )*
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

                $(
                    if let Some($elem_on_set) = self.$elem_on_set.as_mut() {
                        #[allow(unused)]
                        (|$name: &mut $elem, $elem_on_set: $elem_on_set_ty| {
                            $on_set_mod
                        })(&mut elem, $elem_on_set);
                    }
                )*

                self.$name = Some(elem);
            }

            pub(super) fn [< take_ $func >](&mut self) -> Option<$elem> {
                $(
                    if let Some($prev) = self.$prev.as_mut() {
                        $prev.as_audio_source_mut().unset_output();
                    }
                )?

                $(
                    if let Some($next) = self.$next.as_mut() {
                        $next.as_audio_sink_mut().unset_input();
                    }
                )?

                self.$name.take().map(|mut elem| {
                    if self.is_running {
                        elem.on_stop();
                    }

                    $(
                        if let Some($elem_on_take) = self.$elem_on_take.as_mut() {
                            #[allow(unused)]
                            (|$name: &mut $elem, $elem_on_take: $elem_on_take_ty| {
                                $on_take_mod
                            })(&mut elem, $elem_on_take);
                        }
                    )*

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
    demux: Option<AudioDemuxer>,
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
            demux: None,
            dec: None,
            sync: None,
            resizer: None,
            mic: None,

            is_running: false,
        }
    }

    add_pipeline_element! {
        @element AudioDemuxer;

        @long_name audio_demuxer;
        @name demux;

        @next dec;
    }

    add_pipeline_element! {
        @element Box<dyn AudioDecoder>;

        @long_name audio_decoder;
        @name dec;

        @prev demux;
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

        @modify_on_set (sync: &mut Synchronizer) => {
            sync.set_virtual_microphone_statistics(mic.provide_statistics());
        };
        @modify_on_take (sync: &mut Synchronizer) => {
            sync.unset_virtual_microphone_statistics();
        };
    }
}

impl Runnable for AudioPipeline {
    fn update(&mut self) -> error::Result<()> {
        self.dec.as_mut().map(Runnable::update);
        self.sync.as_mut().map(Runnable::update);
        self.mic.as_mut().map(Runnable::update);

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
