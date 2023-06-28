use super::*;
use crate::*;

use mueue::*;

pub enum AudioSystemNotification {}

impl Message for AudioSystemNotification {}

pub trait AudioSystemElement: Runnable {
    fn notification_sender(&self) -> MessageSender<AudioSystemNotification>;
    fn connect(&mut self, send: MessageSender<AudioSystemNotification>);

    fn send(&self, msg: AudioSystemNotification) {
        let _ = self.notification_sender().send(Arc::new(msg));
    }
}

impl_as_trait!(audio_system_element -> AudioSystemElement);

pub trait AudioSource: AudioSystemElement + AsAudioSource {
    fn set_output(&mut self, output: DynMessageSender);

    fn chain(&mut self, sink: &mut dyn AudioSink) {
        let (output, input) = unidirectional_queue_dyn();

        self.set_output(output);
        sink.set_input(input);
    }
}

impl_as_trait!(audio_source -> AudioSource);

pub trait AudioSink: AudioSystemElement + AsAudioSink {
    fn set_input(&mut self, input: DynMessageReceiver);
}

impl_as_trait!(audio_sink -> AudioSink);
