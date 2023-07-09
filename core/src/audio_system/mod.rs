pub mod audio_receiver;
pub mod element;
pub mod virtual_microphone;

use audio_receiver::*;
use element::*;
use virtual_microphone::*;

use crate::util::*;
use crate::*;

use std::collections::HashMap;

use mueue::{unidirectional_queue, Message, MessageEndpoint, MessageReceiver, MessageSender};

type AudioSystemEndpoint = MessageEndpoint<AudioSystemControlMessage, AudioSystemMessage>;

#[non_exhaustive]
pub enum AudioSystemMessage {
    Notification(AudioSystemNotification),
}

impl Message for AudioSystemMessage {}

impl From<AudioSystemNotification> for AudioSystemMessage {
    fn from(msg: AudioSystemNotification) -> Self {
        Self::Notification(msg)
    }
}

#[non_exhaustive]
pub enum AudioSystemControlMessage {
    Stop,
}

impl Message for AudioSystemControlMessage {}

pub struct AudioSystem {
    endpoint: Option<AudioSystemEndpoint>,
    notification_receiver: MessageReceiver<AudioSystemNotification>,

    active_audio_receiver_info: AudioReceiverInfo,
    audio_receivers: HashMap<AudioReceiverInfo, Box<dyn AudioReceiver>>,

    active_virtual_mic_info: VirtualMicrophoneInfo,
    virtual_mics: HashMap<VirtualMicrophoneInfo, Box<dyn VirtualMicrophone>>,
}

impl AudioSystem {
    pub fn new(
        audio_receivers: Vec<Box<dyn AudioReceiver>>,
        virtual_mics: Vec<Box<dyn VirtualMicrophone>>,
    ) -> Self {
        let (notification_sender, notification_receiver) = unidirectional_queue();

        let mut audio_receivers = collect_audio_receivers_map(audio_receivers);
        connect_audio_receivers(&mut audio_receivers, notification_sender.clone());
        let active_audio_receiver = choose_best_audio_receiver(&mut audio_receivers);

        let mut virtual_mics = collect_virtual_microphones_map(virtual_mics);
        connect_virtual_microphones(&mut virtual_mics, notification_sender);
        let active_virtual_mic = choose_best_virtual_microphone(&mut virtual_mics);

        active_audio_receiver
            .as_audio_source_mut()
            .chain(active_virtual_mic.as_audio_sink_mut());

        Self {
            endpoint: None,
            notification_receiver,

            active_audio_receiver_info: active_audio_receiver.info(),
            audio_receivers,

            active_virtual_mic_info: active_virtual_mic.info(),
            virtual_mics,
        }
    }
}

impl Component for AudioSystem {
    type Message = AudioSystemMessage;
    type ControlMessage = AudioSystemControlMessage;

    fn endpoint(&self) -> MessageEndpoint<Self::ControlMessage, Self::Message> {
        self.endpoint
            .clone()
            .expect("A message endpoint wasn't set")
    }

    fn connect(&mut self, end: MessageEndpoint<Self::ControlMessage, Self::Message>) {
        self.endpoint = Some(end);
    }
}

impl Runnable for AudioSystem {
    fn update(&mut self, flow: &mut ControlFlow) -> error::Result<()> {
        self.notification_receiver
            .forward(self.endpoint().as_sender().clone());

        self.endpoint()
            .iter()
            .for_each(|msg| msg.handle(self, &mut *flow));

        todo!()
    }
}

fn collect_audio_receivers_map(
    mut audio_receivers: Vec<Box<dyn AudioReceiver>>,
) -> HashMap<AudioReceiverInfo, Box<dyn AudioReceiver>> {
    audio_receivers
        .drain(..)
        .map(|audio_recv| (audio_recv.info(), audio_recv))
        .collect()
}

fn connect_audio_receivers(
    audio_receivers: &mut HashMap<AudioReceiverInfo, Box<dyn AudioReceiver>>,
    notification_sender: MessageSender<AudioSystemNotification>,
) {
    audio_receivers
        .values_mut()
        .for_each(|audio_recv| audio_recv.connect(notification_sender.clone()));
}

// TODO: Change algorithm for choosing the best audio receiver.
fn choose_best_audio_receiver(
    audio_receivers: &mut HashMap<AudioReceiverInfo, Box<dyn AudioReceiver>>,
) -> &mut dyn AudioReceiver {
    &mut **audio_receivers
        .values_mut()
        .next()
        .expect("No audio receivers were provided")
}

fn collect_virtual_microphones_map(
    mut virtual_mics: Vec<Box<dyn VirtualMicrophone>>,
) -> HashMap<VirtualMicrophoneInfo, Box<dyn VirtualMicrophone>> {
    virtual_mics
        .drain(..)
        .map(|virtual_mic| (virtual_mic.info(), virtual_mic))
        .collect()
}

fn connect_virtual_microphones(
    virtual_mics: &mut HashMap<VirtualMicrophoneInfo, Box<dyn VirtualMicrophone>>,
    notification_sender: MessageSender<AudioSystemNotification>,
) {
    virtual_mics
        .values_mut()
        .for_each(|virtual_mic| virtual_mic.connect(notification_sender.clone()));
}

// TODO: Change algorithm for choosing the best virtual microphones.
fn choose_best_virtual_microphone(
    virtual_mics: &mut HashMap<VirtualMicrophoneInfo, Box<dyn VirtualMicrophone>>,
) -> &mut dyn VirtualMicrophone {
    &mut **virtual_mics
        .values_mut()
        .next()
        .expect("No virtual microphones were provided")
}

crate::impl_control_message_handler! {
    @concrete_component AudioSystem;
    @message AudioSystemMessage;
    @control_message AudioSystemControlMessage;
}
