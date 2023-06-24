pub mod audio_filter;
pub mod audio_receiver;
pub mod virtual_microphone;

//use audio_filter::*;
use audio_receiver::*;
use virtual_microphone::*;

use crate::controller::AudioSystemControlMessage;

use std::collections::HashMap;
use std::sync::Arc;

use mueue::{Message, MessageEndpoint};

type ControllerEndpoint = MessageEndpoint<AudioSystemControlMessage, AudioSystemNotification>;

pub enum AudioSystemNotification {}

impl Message for AudioSystemNotification {}

pub struct AudioSystem {
    controller_end: Option<ControllerEndpoint>,

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
        let active_audio_processor_info = audio_processors[0].info();
        let mut audio_processors = audio_processors
            .drain(..)
            .map(|audio_proc| (audio_proc.info(), audio_proc))
            .collect::<HashMap<_, _>>();

        let active_virtual_mic_info = virtual_mics[0].info();
        let mut virtual_mics = virtual_mics
            .drain(..)
            .map(|virtual_mic| (virtual_mic.info(), virtual_mic))
            .collect::<HashMap<_, _>>();

        let active_audio_processor = audio_processors
            .get_mut(&active_audio_processor_info)
            .unwrap();
        let active_virtual_mic = virtual_mics.get_mut(&active_virtual_mic_info).unwrap();

        active_audio_processor.connect_to(active_virtual_mic.as_audio_filter_mut());

        Self {
            controller_end: None,

            active_audio_processor_info,
            audio_processors,

            active_virtual_mic_info,
            virtual_mics,
        }
    }

    pub fn controller_endoint(&self) -> ControllerEndpoint {
        self.controller_end.clone().unwrap()
    }

    pub fn connect_controller(&mut self, end: ControllerEndpoint) {
        self.controller_end = Some(end.clone());

        self.audio_processors
            .values_mut()
            .for_each(|audio_proc| audio_proc.connect_controller(end.clone()));
        self.virtual_mics
            .values_mut()
            .for_each(|virtual_mic| virtual_mic.connect_controller(end.clone()));
    }

    pub fn send_message(&self, msg: AudioSystemNotification) {
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
