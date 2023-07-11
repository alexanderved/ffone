use crate::error;

use std::mem::ManuallyDrop;
use std::ptr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFlow {
    Continue,
    Break,
}

pub trait Runnable: AsRunnable {
    fn update(&mut self, control_flow: &mut ControlFlow) -> error::Result<()>;

    fn on_start(&mut self) -> error::Result<()> {
        Ok(())
    }

    fn on_stop(&mut self) -> error::Result<()> {
        Ok(())
    }

    fn run(&mut self) -> error::Result<()> {
        let mut state_machine = RunnableStateMachine::new_running(self).map_err(|(_, err)| err)?;
        while state_machine.proceed().is_some() {}

        Ok(())
    }
}

impl<T: Runnable + ?Sized> Runnable for &'_ mut T {
    fn update(&mut self, control_flow: &mut ControlFlow) -> error::Result<()> {
        (**self).update(control_flow)
    }

    fn on_start(&mut self) -> error::Result<()> {
        (**self).on_start()
    }

    fn on_stop(&mut self) -> error::Result<()> {
        (**self).on_stop()
    }

    fn run(&mut self) -> error::Result<()> {
        (**self).run()
    }
}

impl<T: Runnable + ?Sized> Runnable for Box<T> {
    fn update(&mut self, control_flow: &mut ControlFlow) -> error::Result<()> {
        (**self).update(control_flow)
    }

    fn on_start(&mut self) -> error::Result<()> {
        (**self).on_start()
    }

    fn on_stop(&mut self) -> error::Result<()> {
        (**self).on_stop()
    }

    fn run(&mut self) -> error::Result<()> {
        (**self).run()
    }
}

crate::impl_as_trait!(runnable -> Runnable);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum RunnableState {
    #[default]
    NotRunning,
    Running(ControlFlow),
}

pub struct RunnableStateMachine<R: Runnable> {
    state: RunnableState,
    runnable: R,
}

impl<R: Runnable> RunnableStateMachine<R> {
    pub fn new(runnable: R) -> Self {
        Self {
            state: RunnableState::NotRunning,
            runnable: runnable,
        }
    }

    pub fn new_running(mut runnable: R) -> Result<Self, (R, error::Error)> {
        match runnable.on_start() {
            Ok(()) => (),
            Err(err) => return Err((runnable, err)),
        }

        Ok(Self {
            state: RunnableState::Running(ControlFlow::Continue),
            runnable: runnable,
        })
    }

    pub fn start(&mut self) -> error::Result<()> {
        if matches!(self.state, RunnableState::Running(_)) {
            return Err(error::Error::WrongRunnableState);
        }
        self.state = RunnableState::Running(ControlFlow::Continue);

        self.runnable.on_start()?;

        Ok(())
    }

    pub fn stop(&mut self) -> error::Result<()> {
        if matches!(self.state, RunnableState::NotRunning) {
            return Err(error::Error::WrongRunnableState);
        }
        self.state = RunnableState::NotRunning;
        self.runnable.on_stop()?;

        Ok(())
    }

    pub fn next_state(&mut self) -> error::Result<()> {
        match self.state {
            RunnableState::NotRunning => self.start(),
            RunnableState::Running(_) => self.stop(),
        }
    }

    pub fn proceed(&mut self) -> Option<error::Result<()>> {
        if let RunnableState::Running(ref mut control_flow) = self.state {
            if matches!(control_flow, ControlFlow::Continue) {
                return Some(self.runnable.update(control_flow));
            }
        }

        None
    }

    pub fn as_runnable(&self) -> &R {
        &self.runnable
    }

    pub fn as_runnable_mut(&mut self) -> &mut R {
        &mut self.runnable
    }

    pub fn into_runnable(self) -> R {
        let mut manually_drop_self = ManuallyDrop::new(self);
        let _ = manually_drop_self.stop();

        // SAFETY: `self.runnable` is not used after `ptr::read`.
        unsafe { ptr::read(&manually_drop_self.runnable) }
    }
}

impl<R: Runnable> std::ops::Drop for RunnableStateMachine<R> {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
