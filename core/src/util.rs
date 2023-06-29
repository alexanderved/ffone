use crate::*;

use std::sync::Arc;

use mueue::*;

pub trait Component {
    type Message: Message;
    type ControlMessage: Message;

    fn endpoint(&self) -> MessageEndpoint<Self::ControlMessage, Self::Message>;
    fn connect(&mut self, end: MessageEndpoint<Self::ControlMessage, Self::Message>);

    fn send(&self, msg: Self::Message) {
        let _ = self.endpoint().send(Arc::new(msg));
    }
}

pub trait Runnable: AsRunnable {
    fn update(&mut self);

    fn run(&mut self) {
        loop {
            self.update();
        }
    }
}

impl_as_trait!(runnable -> Runnable);

#[doc(hidden)]
#[macro_export]
macro_rules! impl_as_trait {
    ($method:ident -> $trait:ident) => {
        ::paste::paste! {
            pub trait [<As $trait>] {
                fn [<as_ $method>] (&self) -> &dyn $trait;
                fn [<as_ $method _mut>] (&mut self) -> &mut dyn $trait;
                fn [<as_ $method _box>] (
                    self: ::std::boxed::Box<Self>
                ) -> ::std::boxed::Box<dyn $trait>
                where
                    Self: 'static;
            }

            impl<T: $trait> [<As $trait>] for T {
                fn [<as_ $method>] (&self) -> &dyn $trait {
                    self
                }

                fn [<as_ $method _mut>] (&mut self) -> &mut dyn $trait {
                    self
                }

                fn [<as_ $method _box>] (
                    self: ::std::boxed::Box<Self>
                ) -> ::std::boxed::Box<dyn $trait>
                where
                    Self: 'static
                {
                    self
                }
            }
        }
    };
}
