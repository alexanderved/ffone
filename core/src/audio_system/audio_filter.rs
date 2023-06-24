use super::*;

use mueue::*;

pub trait AudioFilter: AsAudioFilter {
    fn controller_endpoint(&self) -> ControllerEndpoint;
    fn connect_controller(&mut self, end: ControllerEndpoint);

    fn set_message_input(&mut self, input: DynMessageReceiver);
    fn set_message_output(&mut self, output: DynMessageSender);

    fn update(&mut self);

    fn send_message(&self, msg: AudioSystemNotification) {
        let _ = self.controller_endpoint().send(Arc::new(msg));
    }

    fn connect_to(&mut self, other: &mut dyn AudioFilter) {
        let (output, input) = unidirectional_queue_dyn();

        self.set_message_output(output);
        other.set_message_input(input);
    }
}

pub trait AsAudioFilter {
    fn as_audio_filter(&self) -> &dyn AudioFilter;
    fn as_audio_filter_mut(&mut self) -> &mut dyn AudioFilter;
}

impl<F: AudioFilter> AsAudioFilter for F {
    fn as_audio_filter(&self) -> &dyn AudioFilter {
        self
    }

    fn as_audio_filter_mut(&mut self) -> &mut dyn AudioFilter {
        self
    }
}
