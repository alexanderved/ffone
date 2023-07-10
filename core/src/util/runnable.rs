use crate::error;

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
        let mut state_machine = RunnableStateMachine::new(self)?;
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

pub struct RunnableStateMachine<R: Runnable> {
    control_flow: ControlFlow,
    runnable: Option<R>,
}

impl<R: Runnable> RunnableStateMachine<R> {
    pub fn new(mut runnable: R) -> error::Result<Self> {
        runnable.on_start()?;

        Ok(Self {
            control_flow: ControlFlow::Continue,
            runnable: Some(runnable),
        })
    }

    pub fn proceed(&mut self) -> Option<error::Result<()>> {
        if matches!(self.control_flow, ControlFlow::Continue) {
            return Some(
                self.runnable
                    .as_mut()
                    .expect("No runnable was found")
                    .update(&mut self.control_flow),
            );
        }

        None
    }

    pub fn stop(&mut self) -> error::Result<()> {
        self.control_flow = ControlFlow::Break;
        
        let Some(runnable) = self.runnable.as_mut() else {
            return Ok(());
        };
        runnable.on_stop()?;

        Ok(())
    }

    pub fn into_runnable(mut self) -> R {
        let _ = self.stop();
        self.runnable.take().unwrap()
    }
}

impl<R: Runnable> std::ops::Drop for RunnableStateMachine<R> {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}