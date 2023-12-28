use crate::audio_system::audio::*;
use crate::audio_system::element::*;
use crate::util::ClockTime;
use crate::util::RunnableStateMachine;
use crate::util::SlaveClock;

use std::cell::Cell;
use std::rc::Rc;

pub type VirtualMicrophoneStateMachine = RunnableStateMachine<Box<dyn VirtualMicrophone>>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtualMicrophoneInfo {
    pub name: String,
}

#[derive(Clone)]
pub struct VirtualMicrophoneStatistics {
    clock: Option<Rc<dyn SlaveClock>>,
    available_duration: Rc<Cell<ClockTime>>,
}

impl VirtualMicrophoneStatistics {
    pub fn new(
        clock: Option<Rc<dyn SlaveClock>>,
        available_duration: Rc<Cell<ClockTime>>
    ) -> Self {
        Self {
            clock,
            available_duration,
        }
    }

    pub fn clock(&self) -> Option<&dyn SlaveClock> {
        self.clock.as_deref()
    }

    pub fn available_duration(&self) -> ClockTime {
        self.available_duration.get()
    }
}

pub trait VirtualMicrophone: AudioSink<RawAudioBuffer> {
    fn info(&self) -> VirtualMicrophoneInfo;

    fn provide_statistics(&self) -> VirtualMicrophoneStatistics;
}

crate::trait_alias!(pub VirtualMicrophoneBuilder:
    AudioSystemElementBuilder<Element = dyn VirtualMicrophone>);
