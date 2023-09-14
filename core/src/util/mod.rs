mod clock;
mod component;
mod element;
mod ring_buffer;
mod runnable;

pub use clock::*;
pub use component::*;
pub use element::*;
pub use ring_buffer::*;
pub use runnable::*;

use std::ptr;

pub fn vec_truncate_front<T>(vec: &mut Vec<T>, len: usize) {
    if len == 0 {
        return;
    }
    if len > vec.len() {
        vec.clear();

        return;
    }

    let remaining_len = vec.len() - len;
    let vec_ptr = vec.as_mut_ptr();

    unsafe {
        let slice = ptr::slice_from_raw_parts_mut(vec_ptr, len);
        ptr::drop_in_place(slice);

        if remaining_len > 0 {
            ptr::copy(vec_ptr.add(len), vec_ptr, remaining_len);
        }
        vec.set_len(remaining_len);
    }
}

pub fn vec_prepend_iter<T, I>(vec: &mut Vec<T>, iter: I)
where
    I: Iterator<Item = T> + ExactSizeIterator,
{
    let len = iter.len();
    if len == 0 {
        return;
    }
    vec.reserve(len);

    let vec_len = vec.len();
    let vec_start = vec.as_mut_ptr();
    let mut vec_cursor = vec_start;
    unsafe {
        ptr::copy(vec_start, vec_start.add(len), vec_len);

        for value in iter {
            vec_cursor.write(value);

            vec_cursor = vec_cursor.add(1);
        }

        vec.set_len(vec_len + len);
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
