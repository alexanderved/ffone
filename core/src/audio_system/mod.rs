pub mod audio_filter;
pub mod audio_processor;
pub mod virtual_microphone;

use audio_processor::*;
use virtual_microphone::*;

use std::collections::HashMap;
use std::sync::Arc;

use mueue::{Message, MessageEndpoint, MessageSender, MessageReceiver, unidirectional_queue};

pub enum AudioSystemMessage {

}

impl Message for AudioSystemMessage {}

pub struct AudioSystem {
    controller_end: Option<MessageEndpoint>,

    audio_message_output: MessageSender,
    audio_message_input: MessageReceiver,

    active_audio_processor_info: AudioProcessorInfo,
    audio_processors: HashMap<AudioProcessorInfo, Box<dyn AudioProcessor>>,

    active_virtual_mic_info: VirtualMicrophoneInfo,
    virtual_mics: HashMap<VirtualMicrophoneInfo, Box<dyn VirtualMicrophone>>,
}

impl AudioSystem {
    pub fn new(
        mut audio_processors: Vec<Box<dyn AudioProcessor>>,
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

        let (audio_message_output, audio_processor_input) = unidirectional_queue();
        let (virtual_mic_output, audio_message_input) = unidirectional_queue();

        let active_audio_processor = audio_processors
            .get_mut(&active_audio_processor_info)
            .unwrap();
        let active_virtual_mic = virtual_mics
            .get_mut(&active_virtual_mic_info)
            .unwrap();

        active_audio_processor.set_message_input(audio_processor_input);
        active_audio_processor.connect_to(active_virtual_mic.as_audio_filter_mut());
        active_virtual_mic.set_message_output(virtual_mic_output);

        Self {
            controller_end: None,

            audio_message_output,
            audio_message_input,

            active_audio_processor_info,
            audio_processors,

            active_virtual_mic_info,
            virtual_mics,
        }
    }

    pub fn controller_endoint(&self) -> MessageEndpoint {
        self.controller_end.clone().unwrap()
    }

    pub fn connect_controller(&mut self, end: MessageEndpoint) {
        self.controller_end = Some(end.clone());

        self.audio_processors
            .values_mut()
            .for_each(|audio_proc| audio_proc.connect_controller(end.clone()));
        self.virtual_mics
            .values_mut()
            .for_each(|virtual_mic| virtual_mic.connect_controller(end.clone()));
    }

    pub fn send_message(&self, msg: AudioSystemMessage) {
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