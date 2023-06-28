#[doc(hidden)]
#[macro_export]
macro_rules! impl_as_trait {
    ($method:ident -> $trait:ident) => {
        ::paste::paste! {
            pub trait [<As $trait>] {
                fn [<as_ $method>] (&self) -> &dyn $trait;
                fn [<as_ $method _mut>] (&mut self) -> &mut dyn $trait;
            }

            impl<T: $trait> [<As $trait>] for T {
                fn [<as_ $method>] (&self) -> &dyn $trait {
                    self
                }

                fn [<as_ $method _mut>] (&mut self) -> &mut dyn $trait {
                    self
                }
            }
        }
    };
}
