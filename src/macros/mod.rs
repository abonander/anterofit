//! Macros for Anterofit.

#[macro_use]
mod request;

/// Define a service trait whose methods make HTTP requests.
///
/// ##Example
/// ```rust,ignore
///
/// service! {
///     pub trait MyService {
///         get! {
///             /// Get the version of this API.
///             fn api_version(&self) -> String {
///                 url = "/version"
///             }
///         }
///
///         post! {
///             /// Register a user with the API.
///             fn register(&self, username: &str, password: &str) {
///                 url = "/register"
///                 fields! {
///                     "username": username,
///                     "password": password,
///                 }
///             }
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! service {
    (
        pub trait $servicenm:ident {
            $(
                #[$verb:ident($($urlpart:tt)+)]
                $(#[$meta:meta])*
                fn $fnname:ident $(<$($generics:tt)*>)* (&self $($args:tt)*) -> $ret:ty
                $(where $($whereclause:tt)+)* $({
                    $($body:tt)+
                })* $(;)*
            )*
        }
    ) => (
        pub trait $servicenm {
            $(
                $(#[$meta:meta])*
                fn $fnname $(<$($generics)*>)* (&self $($args)*) -> $crate::net::Request<$ret>
                $(where $($whereclause:tt)+)*;
            )*
        }

        impl<T: $crate::net::SerializeAdapter> $servicenm for T {
            $(
                fn $fnname $(<$($generics)*>)* (&self $($args)*) -> $crate::net::Request<$ret>
                $(where $($whereclause:tt)+)* {
                        request_impl! {
                            self; $verb; url($($urlpart)+)
                            $(; $($body)+)*
                        }
                }
            )*
        }
    )
}

#[macro_export]
#[doc(hidden)]
macro_rules! method {
    (
        #[$verb:ident($($urlpart:tt)+)]
        $(#[$meta:meta])*
        fn $fnname:ident $(<$($generics:tt)*>)* (&self $($args:tt)*) -> $ret:ty
    ) => ();
}