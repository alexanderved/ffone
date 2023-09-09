use crate::util::*;
use crate::*;

use mueue::*;

#[non_exhaustive]
pub enum AudioSystemElementMessage {
    Error(error::Error),
}

impl Message for AudioSystemElementMessage {}

trait_alias!(@upcast AsAudioSystemElement pub AudioSystemElement:
    Element<Message = AudioSystemElementMessage> + Runnable);

impl_as_trait!(audio_system_element -> AudioSystemElement);

pub trait AudioSystemElementBuilder: ElementBuilder
where
    Self::Element: AudioSystemElement,
{
}

impl<B: ElementBuilder> AudioSystemElementBuilder for B where Self::Element: AudioSystemElement {}

pub trait AudioSource<Out: Message>: AudioSystemElement + AsAudioSource<Out> {
    fn output(&self) -> Option<MessageSender<Out>>;
    fn set_output(&mut self, output: MessageSender<Out>);
    fn unset_output(&mut self);

    fn chain(&mut self, sink: &mut dyn AudioSink<Out>) {
        let (output, input) = unidirectional_queue();

        self.set_output(output);
        sink.set_input(input);
    }
}

impl_as_trait!(audio_source -> AudioSource<Out: Message>);

pub trait AudioSink<In: Message>: AudioSystemElement + AsAudioSink<In> {
    fn input(&self) -> Option<MessageReceiver<In>>;
    fn set_input(&mut self, input: MessageReceiver<In>);
    fn unset_input(&mut self);
}

impl_as_trait!(audio_sink -> AudioSink<In: Message>);

pub trait AudioFilter<In: Message, Out: Message>:
    AudioSink<In> + AudioSource<Out> + AsAudioFilter<In, Out>
{
}

impl_as_trait!(audio_filter -> AudioFilter<In: Message, Out: Message>);
