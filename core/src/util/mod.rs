mod component;
mod runnable;

pub use component::*;
pub use runnable::*;

#[doc(hidden)]
#[macro_export]
macro_rules! impl_as_trait {
    ($method:ident -> $trait:ident $(< $( $generics:tt $(: $bound:tt $(+ $bounds:tt)* )? ),+ >)? ) =>
    {
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
macro_rules! impl_control_message_handler {
    (
        $( @component $comp:ident )? $( @concrete_component $ccomp:ident )?
            $(< $( $cgenerics:tt $(: $cbound:tt $(+ $cbounds:tt)* )? ),+ >)?;

        @message $msg:ident $(< $( $mgenerics:tt $(: $mbound:tt $(+ $mbounds:tt)* )? ),+ >)?;
        @control_message
            $cmsg:ident $(< $( $cmgenerics:tt $(: $cmbound:tt $(+ $cmbounds:tt)* )? ),+ >)?;

        $(
            $op:ident $(( $( $ufields:ident ),* ))? $({ $( $nfields:ident ),* })?
            => $method:ident
            $( =>
                $( $res:ident )?
                $( @map($map_ok:ident) )?
                $( @map_or_else($err:ident, $ok:ident) )?
            )?;
        )*
    ) => {
        impl $(< $( $cmgenerics $(: $cmbound $(+ $cmbounds)* )? ),+ >)? $cmsg $(<
            $( $cmgenerics:tt ),+
        >)? {
            #[allow(unused)]
            pub fn handle $( $(< $( $cgenerics $(: $cbound $(+ $cbounds)* )? ),+ >)? )? (
                self,
                handler: $( &mut impl $comp )? $( &mut $ccomp )? $(< $( $cgenerics ),+ >)?,
                control_flow: &mut $crate::util::ControlFlow,
            ) {
                match self {
                    $(
                        $cmsg::$op $(( $( $ufields ),* ))? $({ $( $nfields ),* })? => {
                            let op_res = handler.$method(
                                $( $( $ufields ),* , )? $( $( $nfields ),* )?
                            );
                            $(
                                $(
                                    let msg = $msg::$res(op_res);
                                    handler.send(msg);
                                )?

                                $(
                                    op_res.map(|op_res| {
                                        handler.send($msg::$map_ok(op_res));
                                    });
                                )?

                                $(
                                    let msg = op_res.map_or_else(
                                        $msg::$err,
                                        $msg::$ok,
                                    );
                                    handler.send(msg);
                                )?
                            )?
                        }
                    )*
                    $cmsg::Stop => {
                        *control_flow = $crate::util::ControlFlow::Break;
                    }
                    _ => {}
                }
            }
        }
    };
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