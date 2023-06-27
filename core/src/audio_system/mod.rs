pub mod element;
pub mod audio_receiver;
pub mod virtual_microphone;

use audio_receiver::*;
use virtual_microphone::*;
use element::*;

use crate::controller::AudioSystemControlMessage;

use std::collections::HashMap;
use std::sync::Arc;

use mueue::{Message, MessageEndpoint, MessageReceiver, unidirectional_queue};

type ControllerEndpoint = MessageEndpoint<AudioSystemControlMessage, AudioSystemMessage>;

pub enum AudioSystemMessage {
    Notification(AudioSystemNotification),
}

impl Message for AudioSystemMessage {}

pub struct AudioSystem {
    controller_end: Option<ControllerEndpoint>,
    notifications_receiver: MessageReceiver<AudioSystemNotification>,

    active_audio_processor_info: AudioReceiverInfo,
    audio_processors: HashMap<AudioReceiverInfo, Box<dyn AudioReceiver>>,

    active_virtual_mic_info: VirtualMicrophoneInfo,
    virtual_mics: HashMap<VirtualMicrophoneInfo, Box<dyn VirtualMicrophone>>,
}

impl AudioSystem {
    pub fn new(
        mut audio_processors: Vec<Box<dyn AudioReceiver>>,
        mut virtual_mics: Vec<Box<dyn VirtualMicrophone>>,
    ) -> Self {
        let (notifications_sender, notifications_receiver) = unidirectional_queue();

        let active_audio_processor_info = audio_processors[0].info();
        let mut audio_processors = audio_processors
            .drain(..)
            .map(|mut audio_proc| {
                audio_proc.connect(notifications_sender.clone());
                audio_proc
            })
            .map(|audio_proc| (audio_proc.info(), audio_proc))
            .collect::<HashMap<_, _>>();

        let active_virtual_mic_info = virtual_mics[0].info();
        let mut virtual_mics = virtual_mics
            .drain(..)
            .map(|mut virtual_mic| {
                virtual_mic.connect(notifications_sender.clone());
                virtual_mic
            })
            .map(|virtual_mic| (virtual_mic.info(), virtual_mic))
            .collect::<HashMap<_, _>>();

        let active_audio_processor = audio_processors
            .get_mut(&active_audio_processor_info)
            .unwrap();
        let active_virtual_mic = virtual_mics.get_mut(&active_virtual_mic_info).unwrap();

        active_audio_processor
            .as_audio_source_mut()
            .chain(active_virtual_mic.as_audio_sink_mut());

        Self {
            controller_end: None,
            notifications_receiver,

            active_audio_processor_info,
            audio_processors,

            active_virtual_mic_info,
            virtual_mics,
        }
    }

    pub fn controller_endoint(&self) -> ControllerEndpoint {
        self.controller_end.clone().unwrap()
    }

    pub fn connect(&mut self, end: ControllerEndpoint) {
        self.controller_end = Some(end.clone());
    }

    pub fn send(&self, msg: AudioSystemMessage) {
        let _ = self.controller_endoint().send(Arc::new(msg));
    }

    pub fn update(&mut self) {

    }

    pub fn run(&mut self) {
        loop {
            self.update();
        }
    }
}
