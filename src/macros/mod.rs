//! Macros for Anterofit.

mod method;
mod request;

#[macro_export]
macro_rules! service {
    (
        pub trait $servicenm:ident {
            $($methods:tt)*
        }
    ) => (
        pub trait $servicenm : $crate::net::RequestAdapter {
            $($methods)*
        }

        impl<T: $crate::net::RequestAdapter> $servicenm for T {}
    )
}