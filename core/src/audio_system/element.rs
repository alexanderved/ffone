use crate::util::*;
use crate::*;

use mueue::*;

#[non_exhaustive]
pub enum AudioSystemNotification {}

impl Message for AudioSystemNotification {}

trait_alias!(pub AudioSystemElement:
    Element<Notiication = AudioSystemNotification> + Runnable + AsAudioSystemElement);

impl_as_trait!(audio_system_element -> AudioSystemElement);

pub trait AudioSystemElementBuilder: ElementBuilder
where
    Self::Element: AudioSystemElement,
{
}

impl<B: ElementBuilder> AudioSystemElementBuilder for B where Self::Element: AudioSystemElement {}

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
