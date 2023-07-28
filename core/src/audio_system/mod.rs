pub mod audio;
pub mod audio_decoder;
pub mod element;
pub mod virtual_microphone;
mod sync;

use audio_decoder::*;
use element::*;
use virtual_microphone::*;
use sync::*;

use crate::util::*;
use crate::*;

use std::collections::HashMap;

use mueue::{unidirectional_queue, Message, MessageEndpoint, MessageReceiver, MessageSender};

pub type AudioSystemEndpoint = MessageEndpoint<AudioSystemControlMessage, AudioSystemMessage>;

#[non_exhaustive]
pub enum AudioSystemMessage {}

impl Message for AudioSystemMessage {}

#[non_exhaustive]
pub enum AudioSystemControlMessage {
    Stop,
}

impl Message for AudioSystemControlMessage {}

pub struct AudioSystem {
    endpoint: AudioSystemEndpoint,
    notification_recv: MessageReceiver<AudioSystemElementMessage>,

    active_audio_dec: Option<AudioDecoderStateMachine>,
    audio_decs: HashMap<AudioDecoderInfo, Box<dyn AudioDecoder>>,

    active_virtual_mic: Option<VirtualMicrophoneStateMachine>,
    virtual_mics: HashMap<VirtualMicrophoneInfo, Box<dyn VirtualMicrophone>>,
}

impl AudioSystem {
    pub fn new(
        end: AudioSystemEndpoint,
        audio_decs_builders: Vec<Box<dyn AudioDecoderBuilder>>,
        virtual_mics_builders: Vec<Box<dyn VirtualMicrophoneBuilder>>,
    ) -> Self {
        let (notification_sender, notification_receiver) = unidirectional_queue();

        let audio_decs = collect_audio_decs(audio_decs_builders, notification_sender.clone());
        let virtual_mics =
            collect_virtual_microphones(virtual_mics_builders, notification_sender.clone());

        Self {
            endpoint: end,
            notification_recv: notification_receiver,

            active_audio_dec: None,
            audio_decs,

            active_virtual_mic: None,
            virtual_mics,
        }
    }
}

fn collect_audio_decs(
    audio_decs_builders: Vec<Box<dyn AudioDecoderBuilder>>,
    notification_sender: MessageSender<AudioSystemElementMessage>,
) -> HashMap<AudioDecoderInfo, Box<dyn AudioDecoder>> {
    audio_decs_builders
        .into_iter()
        .map(|mut builder| {
            builder.set_sender(notification_sender.clone());
            builder
        })
        .filter_map(|builder| builder.build().ok())
        .map(|audio_dec| (audio_dec.info(), audio_dec))
        .collect()
}

fn collect_virtual_microphones(
    virtual_mics_builders: Vec<Box<dyn VirtualMicrophoneBuilder>>,
    notification_sender: MessageSender<AudioSystemElementMessage>,
) -> HashMap<VirtualMicrophoneInfo, Box<dyn VirtualMicrophone>> {
    virtual_mics_builders
        .into_iter()
        .map(|mut builder| {
            builder.set_sender(notification_sender.clone());
            builder
        })
        .filter_map(|builder| builder.build().ok())
        .map(|virtual_mic| (virtual_mic.info(), virtual_mic))
        .collect()
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

crate::impl_control_message_handler! {
    @concrete_component AudioSystem;
    @message AudioSystemMessage;
    @control_message AudioSystemControlMessage;
}

impl Runnable for AudioSystem {
    fn update(&mut self, flow: &mut ControlFlow) -> error::Result<()> {
        self.endpoint()
            .iter()
            .for_each(|msg| msg.handle(self, &mut *flow));

        todo!()
    }

    fn on_start(&mut self) {
        self.active_audio_dec = Some(choose_best_audio_decoder(&mut self.audio_decs));
        self.active_virtual_mic = Some(choose_best_virtual_microphone(&mut self.virtual_mics));

        todo!("Chain the audio decoder and the virtual mic")
    }
}

fn choose_best_audio_decoder(
    audio_decs: &mut HashMap<AudioDecoderInfo, Box<dyn AudioDecoder>>,
) -> AudioDecoderStateMachine {
    let mut audio_decs_iter = audio_decs.drain();
    let active_audio_dec = audio_decs_iter
        .next()
        .map(|(_, dec)| RunnableStateMachine::new_running(dec));
    *audio_decs = audio_decs_iter.collect();

    active_audio_dec.expect("No suitable audio receivers were provided")
}

fn choose_best_virtual_microphone(
    virtual_mics: &mut HashMap<VirtualMicrophoneInfo, Box<dyn VirtualMicrophone>>,
) -> VirtualMicrophoneStateMachine {
    let mut virtual_mics_iter = virtual_mics.drain();
    let active_virtual_mic = virtual_mics_iter
        .next()
        .map(|(_, mic)| RunnableStateMachine::new_running(mic));
    *virtual_mics = virtual_mics_iter.collect();

    active_virtual_mic.expect("No suitable audio receivers were provided")
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
