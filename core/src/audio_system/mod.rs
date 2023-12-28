pub mod audio;
pub mod element;
pub mod pipeline;
pub mod queue;

use element::*;
use pipeline::*;
use pipeline::{audio_decoder::*, resizer::*, sync::*, virtual_microphone::*};

use crate::util::*;
use crate::*;

use std::collections::HashMap;
use std::sync::Arc;

use mueue::{unidirectional_queue, Message, MessageEndpoint, MessageReceiver, MessageSender};

use self::pipeline::demuxer::AudioDemuxer;

pub type AudioSystemEndpoint = MessageEndpoint<AudioSystemControlMessage, AudioSystemMessage>;

#[non_exhaustive]
pub enum AudioSystemMessage {}

impl Message for AudioSystemMessage {}

#[non_exhaustive]
pub enum AudioSystemControlMessage {
    Stop,
}

impl Message for AudioSystemControlMessage {}

#[allow(dead_code)]
pub struct AudioSystem {
    endpoint: AudioSystemEndpoint,
    notification_recv: MessageReceiver<AudioSystemElementMessage>,

    pipeline: AudioPipelineStateMachine,

    audio_decs: HashMap<AudioDecoderInfo, Option<Box<dyn AudioDecoder>>>,
    virtual_mics: HashMap<VirtualMicrophoneInfo, Option<Box<dyn VirtualMicrophone>>>,
}

impl AudioSystem {
    pub fn new(
        end: AudioSystemEndpoint,
        audio_decs_builders: Vec<Box<dyn AudioDecoderBuilder>>,
        virtual_mics_builders: Vec<Box<dyn VirtualMicrophoneBuilder>>,
    ) -> Self {
        let (notification_send, notification_recv) = unidirectional_queue();

        let sys_clock = Arc::new(SystemClock::new());

        let demux = AudioDemuxer::new(notification_send.clone());
        let mut audio_decs = collect_audio_decs(audio_decs_builders, notification_send.clone());
        let sync = Synchronizer::new(notification_send.clone(), sys_clock);
        let resizer = AudioResizer::new(notification_send.clone());
        let mut virtual_mics =
            collect_virtual_microphones(virtual_mics_builders, notification_send);

        let mut pipeline = AudioPipeline::new();
        pipeline.set_audio_demuxer(demux);
        pipeline.set_audio_decoder(take_first_audio_decoder(&mut audio_decs));
        pipeline.set_synchronizer(sync);
        pipeline.set_resizer(resizer);
        pipeline.set_virtual_microphone(take_first_virtual_microphone(&mut virtual_mics));

        Self {
            endpoint: end,
            notification_recv,

            pipeline: RunnableStateMachine::new(pipeline),

            audio_decs,
            virtual_mics,
        }
    }

    pub fn choose_audio_decoder(&mut self, info: AudioDecoderInfo) {
        if let Some(old_dec) = self.pipeline.runnable_mut().take_audio_decoder() {
            self.audio_decs.insert(old_dec.info(), Some(old_dec));
        }

        let dec = self.audio_decs.get_mut(&info).and_then(Option::take);
        if let Some(dec) = dec {
            self.pipeline.runnable_mut().set_audio_decoder(dec);
        }
    }

    pub fn choose_virtual_microphone(&mut self, info: VirtualMicrophoneInfo) {
        if let Some(old_mic) = self.pipeline.runnable_mut().take_virtual_microphone() {
            self.virtual_mics.insert(old_mic.info(), Some(old_mic));
        }

        let mic = self.virtual_mics.get_mut(&info).and_then(Option::take);
        if let Some(mic) = mic {
            self.pipeline.runnable_mut().set_virtual_microphone(mic);
        }
    }
}

fn collect_audio_decs(
    audio_decs_builders: Vec<Box<dyn AudioDecoderBuilder>>,
    notification_sender: MessageSender<AudioSystemElementMessage>,
) -> HashMap<AudioDecoderInfo, Option<Box<dyn AudioDecoder>>> {
    audio_decs_builders
        .into_iter()
        .map(|mut builder| {
            builder.set_sender(notification_sender.clone());
            builder
        })
        .filter_map(|builder| builder.build().ok())
        .map(|audio_dec| (audio_dec.info(), Some(audio_dec)))
        .collect()
}

fn collect_virtual_microphones(
    virtual_mics_builders: Vec<Box<dyn VirtualMicrophoneBuilder>>,
    notification_sender: MessageSender<AudioSystemElementMessage>,
) -> HashMap<VirtualMicrophoneInfo, Option<Box<dyn VirtualMicrophone>>> {
    virtual_mics_builders
        .into_iter()
        .map(|mut builder| {
            builder.set_sender(notification_sender.clone());
            builder
        })
        .filter_map(|builder| builder.build().ok())
        .map(|virtual_mic| (virtual_mic.info(), Some(virtual_mic)))
        .collect()
}

fn take_first_audio_decoder(
    audio_decs: &mut HashMap<AudioDecoderInfo, Option<Box<dyn AudioDecoder>>>,
) -> Box<dyn AudioDecoder> {
    let first_audio_dec_info = audio_decs
        .keys()
        .next()
        .cloned()
        .expect("No audio decoders were provided");
    audio_decs
        .get_mut(&first_audio_dec_info)
        .unwrap()
        .take()
        .expect("The audio decoder was already taken")
}

fn take_first_virtual_microphone(
    virtual_mics: &mut HashMap<VirtualMicrophoneInfo, Option<Box<dyn VirtualMicrophone>>>,
) -> Box<dyn VirtualMicrophone> {
    let first_virtual_mic_info = virtual_mics
        .keys()
        .next()
        .cloned()
        .expect("No virtual microphones were provided");
    virtual_mics
        .get_mut(&first_virtual_mic_info)
        .unwrap()
        .take()
        .expect("The virtual microphone was already taken")
}

impl Component for AudioSystem {
    type Message = AudioSystemMessage;
    type ControlMessage = AudioSystemControlMessage;

    fn endpoint(&self) -> MessageEndpoint<Self::ControlMessage, Self::Message> {
        self.endpoint.clone()
    }

    fn connect(&mut self, end: MessageEndpoint<Self::ControlMessage, Self::Message>) {
        self.endpoint = end;
    }
}

impl Runnable for AudioSystem {
    fn update(&mut self, _flow: Option<&mut ControlFlow>) -> error::Result<()> {
        todo!()
    }

    fn on_start(&mut self) {
        let _ = self.pipeline.start();
    }
}

pub struct AudioSystemBuilder {
    end: Option<AudioSystemEndpoint>,

    audio_decs_builders: Vec<Box<dyn AudioDecoderBuilder>>,
    virtual_mics_builders: Vec<Box<dyn VirtualMicrophoneBuilder>>,
}

impl AudioSystemBuilder {
    pub fn new() -> Self {
        Self {
            end: None,

            audio_decs_builders: vec![],
            virtual_mics_builders: vec![],
        }
    }

    pub fn add_audio_dec<B: AudioDecoderBuilder + 'static>(mut self, builder: B) -> Self {
        self.audio_decs_builders.push(Box::new(builder));
        self
    }

    pub fn add_virtual_microphone<B: VirtualMicrophoneBuilder + 'static>(
        mut self,
        builder: B,
    ) -> Self {
        self.virtual_mics_builders.push(Box::new(builder));
        self
    }
}

impl ComponentBuilder for AudioSystemBuilder {
    type Component = AudioSystem;

    fn set_endpoint(&mut self, end: AudioSystemEndpoint) {
        self.end = Some(end);
    }

    fn build(self: Box<Self>) -> error::Result<Box<Self::Component>> {
        let Self {
            end,
            audio_decs_builders,
            virtual_mics_builders,
        } = *self;
        let end = end.expect("An audio system endpoint wasn't provided");

        Ok(Box::new(AudioSystem::new(
            end,
            audio_decs_builders,
            virtual_mics_builders,
        )))
    }
}
