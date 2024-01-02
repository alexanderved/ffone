use crate::error;

use std::mem::ManuallyDrop;
use std::ptr;

pub trait Runnable: AsRunnable {
    fn update(&mut self) -> error::Result<()>;

    fn on_start(&mut self) {}

    fn on_stop(&mut self) {}

    fn run(&mut self) {
        let mut state_machine = RunnableStateMachine::new_running(self);
        while state_machine.proceed().is_some() {}
    }
}

impl<T: Runnable + ?Sized> Runnable for &'_ mut T {
    fn update(&mut self) -> error::Result<()> {
        (**self).update()
    }

    fn on_start(&mut self) {
        (**self).on_start()
    }

    fn on_stop(&mut self) {
        (**self).on_stop()
    }

    fn run(&mut self) {
        (**self).run()
    }
}

impl<T: Runnable + ?Sized> Runnable for Box<T> {
    fn update(&mut self) -> error::Result<()> {
        (**self).update()
    }

    fn on_start(&mut self) {
        (**self).on_start()
    }

    fn on_stop(&mut self) {
        (**self).on_stop()
    }

    fn run(&mut self) {
        (**self).run()
    }
}

crate::impl_as_trait!(runnable -> Runnable);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum RunnableState {
    #[default]
    NotRunning,
    Running,
}

pub struct RunnableStateMachine<R: Runnable> {
    state: RunnableState,
    runnable: R,
}

impl<R: Runnable> RunnableStateMachine<R> {
    pub fn new(runnable: R) -> Self {
        Self {
            state: RunnableState::NotRunning,
            runnable,
        }
    }

    pub fn new_running(mut runnable: R) -> Self {
        runnable.on_start();

        Self {
            state: RunnableState::Running,
            runnable,
        }
    }

    pub fn start(&mut self) -> error::Result<()> {
        if matches!(self.state, RunnableState::Running) {
            return Err(error::Error::WrongRunnableState);
        }
        self.runnable.on_start();
        self.state = RunnableState::Running;

        Ok(())
    }

    pub fn stop(&mut self) -> error::Result<()> {
        if matches!(self.state, RunnableState::NotRunning) {
            return Err(error::Error::WrongRunnableState);
        }
        self.runnable.on_stop();
        self.state = RunnableState::NotRunning;

        Ok(())
    }

    pub fn next_state(&mut self) -> error::Result<()> {
        match self.state {
            RunnableState::NotRunning => self.start(),
            RunnableState::Running => self.stop(),
        }
    }

    pub fn proceed(&mut self) -> Option<error::Result<()>> {
        if matches!(self.state, RunnableState::Running) {
            return Some(self.runnable.update());
        }

        None
    }

    pub fn runnable(&self) -> &R {
        &self.runnable
    }

    pub fn runnable_mut(&mut self) -> &mut R {
        &mut self.runnable
    }

    pub fn into_runnable(self) -> R {
        let mut manually_drop_self = ManuallyDrop::new(self);
        let _ = manually_drop_self.stop();

        // SAFETY: `self.runnable` is not used after `ptr::read`.
        unsafe { ptr::read(&manually_drop_self.runnable) }
    }

    pub fn is_running(&self) -> bool {
        matches!(self.state, RunnableState::Running)
    }
}

impl<R: Runnable> std::ops::Drop for RunnableStateMachine<R> {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}