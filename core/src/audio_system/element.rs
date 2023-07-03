use super::*;
use crate::*;

use mueue::*;

pub enum AudioSystemNotification {}

impl Message for AudioSystemNotification {}

pub trait AudioSystemElement: Runnable + Send + Sync {
    fn notification_sender(&self) -> MessageSender<AudioSystemNotification>;
    fn connect(&mut self, send: MessageSender<AudioSystemNotification>);

    fn send(&self, msg: AudioSystemNotification) {
        let _ = self.notification_sender().send(msg);
    }
}

impl_as_trait!(audio_system_element -> AudioSystemElement);

pub trait AudioSource: AudioSystemElement + AsAudioSource {
    fn set_output(&mut self, output: MessageSender<Self::Out>);

    fn chain(&mut self, sink: &mut dyn AudioSink<In = Self::Out>) {
        let (output, input) = unidirectional_queue();

        self.set_output(output);
        sink.set_input(input);
    }
}

pub trait AsAudioSource {
    type Out: Message;

    fn as_audio_source(&self) -> &dyn AudioSource<Out = Self::Out>;
    fn as_audio_source_mut(&mut self) -> &mut dyn AudioSource<Out = Self::Out>;
    fn as_audio_source_box(self: Box<Self>) -> Box<dyn AudioSource<Out = Self::Out>>
    where
        Self: 'static;
}

impl<T: AudioSource> AsAudioSource for T {
    type Out = T::Out;

    fn as_audio_source(&self) -> &dyn AudioSource<Out = Self::Out> {
        self
    }

    fn as_audio_source_mut(&mut self) -> &mut dyn AudioSource<Out = Self::Out> {
        self
    }

    fn as_audio_source_box(self: Box<Self>) -> Box<dyn AudioSource<Out = Self::Out>>
    where
        Self: 'static,
    {
        self
    }
}

pub trait AudioSink: AudioSystemElement + AsAudioSink {
    fn set_input(&mut self, input: MessageReceiver<Self::In>);
}

pub trait AsAudioSink {
    type In: Message;

    fn as_audio_sink(&self) -> &dyn AudioSink<In = Self::In>;
    fn as_audio_sink_mut(&mut self) -> &mut dyn AudioSink<In = Self::In>;
    fn as_audio_sink_box(self: Box<Self>) -> Box<dyn AudioSink<In = Self::In>>
    where
        Self: 'static;
}

impl<T: AudioSink> AsAudioSink for T {
    type In = T::In;

    fn as_audio_sink(&self) -> &dyn AudioSink<In = Self::In> {
        self
    }

    fn as_audio_sink_mut(&mut self) -> &mut dyn AudioSink<In = Self::In> {
        self
    }

    fn as_audio_sink_box(self: Box<Self>) -> Box<dyn AudioSink<In = Self::In>>
    where
        Self: 'static,
    {
        self
    }
}
