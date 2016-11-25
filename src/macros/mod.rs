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
///         /// Get the version of this API.
///         #[GET("/version")]
///         fn api_version(&self) -> String {
///             url = "/version"
///         }
///
///
///         /// Register a user with the API.
///         #[POST("/register")]
///         fn register(&self, username: &str, password: &str) {
///             url = "/register"
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
        trait $servicenm:ident {
            $(
                $(#[$meta:meta])*
                fn $fnname:ident $(<$($generics:tt)*>)* (&self $($args:tt)*) -> $ret:ty
                $(where $($whereclause:tt)+)* {
                    $($body:tt)+
                }
            )*
        }
    ) => (
        trait $servicenm {
            $(
                $(#[$meta])*
                fn $fnname $(<$($generics)*>)* (&self $($args)*) -> $crate::net::Request<$ret>
                $(where $($whereclause)+)*;
            )*
        }

        impl<T: $crate::net::SerializeAdapter> $servicenm for T {
            $(
                $(#[$meta])*
                fn $fnname $(<$($generics)*>)* (&self $($args)*) -> $crate::net::Request<$ret>
                $(where $($whereclause)+)* {
                    request_impl! {
                        self; $($body)+
                    }
                }
            )*
        }
    );
    (
        pub trait $servicenm:ident {
            $(
                $(#[$meta:meta])*
                fn $fnname:ident $(<$($generics:tt)*>)* (&self $($args:tt)*) -> $ret:ty
                $(where $($whereclause:tt)+)* {
                    $($body:tt)+
                }
            )*
        }
    ) => (
        pub trait $servicenm {
            $(
                $(#[$meta])*
                fn $fnname $(<$($generics)*>)* (&self $($args)*) -> $crate::net::Request<$ret>
                $(where $($whereclause)+)*;
            )*
        }

        impl<T: $crate::net::SerializeAdapter> $servicenm for T {
            $(
                $(#[$meta])*
                fn $fnname $(<$($generics)*>)* (&self $($args)*) -> $crate::net::Request<$ret>
                $(where $($whereclause)+)* {
                    request_impl! {
                        self; $($body)+
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