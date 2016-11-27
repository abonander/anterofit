//! Macros for Anterofit.

#[macro_use]
mod request;

/// Define a service trait whose methods make HTTP requests.
///
/// ##Example
/// ```rust
/// # #[macro_use] extern crate anterofit;
/// # fn main() {}
/// service! {
///     pub trait MyService {
///         /// Get the version of this API.
///         fn api_version(&self) -> String {
///             GET("/version")
///         }
///
///         /// Register a user with the API.
///         fn register(&self, username: &str, password: &str) {
///             POST("/register");
///             fields! {
///                 username, password
///             }
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! service {
    (
        $(#[$meta:meta])*
        trait $servicenm:ident {
            $(
                $(#[$fnmeta:meta])*
                fn $fnname:ident $(<$($generics:tt)*>)* (&self $($args:tt)*) $(-> $ret:ty)*
                $(where $($whereclause:tt)+)* {
                    $($body:tt)+
                }
            )*
        }
    ) => (
        $(#[$meta])*
        trait $servicenm {
            $(
                $(#[$fnmeta])*
                fn $fnname $(<$($generics)*>)* (&self $($args)*) -> $crate::net::Request<$($ret)*>
                $(where $($whereclause)+)*;
            )*
        }

        impl<T: $crate::net::AbsAdapter> $servicenm for T {
            $(
                fn $fnname $(<$($generics)*>)* (&self $($args)*) -> $crate::net::Request<$($ret)*>
                $(where $($whereclause)+)* {
                    request_impl! {
                        self; $($body)+
                    }
                }
            )*
        }
    );
    (
        $(#[$meta:meta])*
        pub trait $servicenm:ident {
            $(
                $(#[$fnmeta:meta])*
                fn $fnname:ident $(<$($generics:tt)*>)* (&self $($args:tt)*) $(-> $ret:ty)*
                $(where $($whereclause:tt)+)* {
                    $($body:tt)+
                }
            )*
        }
    ) => (
        $(#[$meta])*
        pub trait $servicenm {
            $(
                $(#[$fnmeta])*
                fn $fnname $(<$($generics)*>)* (&self $($args)*) -> $crate::net::Request<$($ret)*>
                $(where $($whereclause)+)*;
            )*
        }

        impl<T: $crate::net::AbsAdapter> $servicenm for T {
            $(
                fn $fnname $(<$($generics)*>)* (&self $($args)*) -> $crate::net::Request<$($ret)*>
                $(where $($whereclause)+)* {
                    request_impl! {
                        self; $($body)+
                    }
                }
            )*
        }
    )
}