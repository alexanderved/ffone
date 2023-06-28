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

pub trait AsAudioSystemElement {
    fn as_audio_system_element(&self) -> &dyn AudioSystemElement;
    fn as_audio_system_element_mut(&mut self) -> &mut dyn AudioSystemElement;
}

impl<E: AudioSystemElement> AsAudioSystemElement for E {
    fn as_audio_system_element(&self) -> &dyn AudioSystemElement {
        self
    }

    fn as_audio_system_element_mut(&mut self) -> &mut dyn AudioSystemElement {
        self
    }
}

pub trait AudioSource: AudioSystemElement + AsAudioSource {
    fn set_output(&mut self, output: DynMessageSender);

    fn chain(&mut self, sink: &mut dyn AudioSink) {
        let (output, input) = unidirectional_queue_dyn();

        self.set_output(output);
        sink.set_input(input);
    }
}

pub trait AsAudioSource {
    fn as_audio_source(&self) -> &dyn AudioSource;
    fn as_audio_source_mut(&mut self) -> &mut dyn AudioSource;
}

impl<S: AudioSource> AsAudioSource for S {
    fn as_audio_source(&self) -> &dyn AudioSource {
        self
    }

    fn as_audio_source_mut(&mut self) -> &mut dyn AudioSource {
        self
    }
}

pub trait AudioSink: AudioSystemElement + AsAudioSink {
    fn set_input(&mut self, input: DynMessageReceiver);
}

pub trait AsAudioSink {
    fn as_audio_sink(&self) -> &dyn AudioSink;
    fn as_audio_sink_mut(&mut self) -> &mut dyn AudioSink;
}

impl<S: AudioSink> AsAudioSink for S {
    fn as_audio_sink(&self) -> &dyn AudioSink {
        self
    }

    fn as_audio_sink_mut(&mut self) -> &mut dyn AudioSink {
        self
    }
}
