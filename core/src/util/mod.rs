mod component;
mod element;
mod runnable;

pub use component::*;
pub use element::*;
pub use runnable::*;

use std::cell::Cell;
use std::time::*;

pub struct Timer {
    pub start: Cell<Instant>,
    interval: Duration,
}

impl Timer {
    pub fn new(interval: Duration) -> Self {
        Self {
            start: Cell::new(Instant::now()),
            interval,
        }
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }

    pub fn set_interval(&mut self, interval: Duration) {
        self.interval = interval;
    }

    pub fn restart(&self) {
        self.start.set(Instant::now());
    }

    pub fn is_time_out(&self) -> bool {
        if self.start.get().elapsed() > self.interval {
            self.start.set(Instant::now());
            return true;
        }

        false
    }
}

#[macro_export]
macro_rules! try_block {
    (
        $( $block:tt )*
    ) => {
        (|| {
            $( $block )*
        })()
    };
}

#[macro_export]
macro_rules! guard {
    (
        @guard $guard:ident $(< $( $generics:tt $(: $bound:tt $(+ $bounds:tt)* )? ),+ >)? {
            $( $field:ident : $field_ty:ty, )*
        }
        @new -> $ret:ty $( where $wrapper:ident (Self))? $new_block:block
        @drop $drop_block:block
        $( @unwrap_with $( $unwrap_op:tt )* )?
    ) => {
        struct $guard $(< $( $generics $(: $bound $(+ $bounds)* )? ),+ >)? {
            $( $field : $field_ty, )*
        }

        impl $(< $( $generics $(: $bound $(+ $bounds)* )? ),+ >)? $guard $(< $( $generics ),+ >)?
        {
            fn new($( mut $field : $field_ty, )*) -> $ret {
                $new_block;

                $( $wrapper )? (Self {
                    $( $field, )*
                })
            }
        }

        impl $(< $( $generics $(: $bound $(+ $bounds)* )? ),+ >)? ::std::ops::Drop for
            $guard $(< $( $generics ),+ >)?
        {
            fn drop(&mut self) {
                let Self { $( $field, )* } = self;
                $drop_block;
            }
        }

        let _guard = $guard::new( $( $field, )* ) $( $( $unwrap_op )* )?;
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_as_trait {
    (
        $method:ident ->
            $trait:ident $(< $( $generics:tt $(: $bound:tt $(+ $bounds:tt)* )? ),+ >)?
    ) => {
        ::paste::paste! {
            pub trait [<As $trait>] $(< $( $generics $(: $bound $(+ $bounds)* )? ),+ >)? {
                fn [<as_ $method>] (&self) -> &dyn $trait $(< $( $generics ),+ >)?;
                fn [<as_ $method _mut>] (&mut self) -> &mut dyn $trait $(< $( $generics ),+ >)?;
                fn [<as_ $method _box>] (
                    self: ::std::boxed::Box<Self>
                ) -> ::std::boxed::Box<dyn $trait $(< $( $generics ),+ >)?>
                where
                    Self: 'static;
            }

            impl<
                $( $( $generics $(: $bound $(+ $bounds)* )?, )+ )?
                T: $trait $(< $( $generics ),+ >)?
            > [<As $trait>] $(< $( $generics ),+ >)? for T {
                fn [<as_ $method>] (&self) -> &dyn $trait $(< $( $generics ),+ >)? {
                    self
                }

                fn [<as_ $method _mut>] (&mut self) -> &mut dyn $trait $(< $( $generics ),+ >)? {
                    self
                }

                fn [<as_ $method _box>] (
                    self: ::std::boxed::Box<Self>
                ) -> ::std::boxed::Box<dyn $trait $(< $( $generics ),+ >)?>
                where
                    Self: 'static
                {
                    self
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! trait_alias {
    ( $( @upcast $upcast:ident )? $vis:vis $alias:ident : $( $trait:tt )*) => {
        $vis trait $alias: $( $trait )* $( + $upcast )? {}

        impl<T: $( $trait )*> $alias for T {}
    };
}
