use super::*;
use crate::*;

use mueue::*;

pub enum AudioSystemNotification {}

impl Message for AudioSystemNotification {}

pub trait AudioSystemElement: Runnable + Send {
    fn notification_sender(&self) -> MessageSender<AudioSystemNotification>;
    fn connect(&mut self, send: MessageSender<AudioSystemNotification>);

    fn send(&self, msg: AudioSystemNotification) {
        let _ = self.notification_sender().send(msg);
    }
}

impl_as_trait!(audio_system_element -> AudioSystemElement);

pub trait AudioSystemElementBuilder {
    type Element: AudioSystemElement + ?Sized;

    fn set_notification_sender(&mut self, send: MessageSender<AudioSystemNotification>);
    fn build(self: Box<Self>) -> error::Result<Box<Self::Element>>;
}

pub trait AudioSource<Out: Message>: AudioSystemElement + AsAudioSource<Out> {
    fn set_output(&mut self, output: MessageSender<Out>);

    fn chain(&mut self, sink: &mut dyn AudioSink<Out>) {
        let (output, input) = unidirectional_queue();

        self.set_output(output);
        sink.set_input(input);
    }
}

impl_as_trait!(audio_source -> AudioSource<Out: Message>);

pub trait AudioSink<In: Message>: AudioSystemElement + AsAudioSink<In> {
    fn set_input(&mut self, input: MessageReceiver<In>);
}

impl_as_trait!(audio_sink -> AudioSink<In: Message>);
