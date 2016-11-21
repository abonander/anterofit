//! Macros for service methods wrapping REST calls.

// It'd be nice to use a macro to avoid copy-pasting but unfortunately that doesn't really work.

/// Create a service method wrapping a GET request.
///
/// ##Note
/// The `body!` and `fields!` macros are not allowed in the body of this request.
///
/// ##Example
/// ```notest
/// service!{
///     pub trait UserService {
///         get! {
///             fn get_user_info(&self, username: &str) -> UserInfo {
///                 url = "/user/{}", username
///             }
///         }
///     }
/// ```
#[macro_export]
macro_rules! get {
    (
        $(#[$meta:meta])*
        fn $fnname:ident $(<$($generics:tt)*>)* (&self $($args:tt)*) -> $ret:ty {
                $($body:tt)+
        }
    ) => (
        $(#[$meta])*
        fn $fnname $(<$($generics)*>)* (&self $($args)*) -> $crate::net::Request<Self, $ret> {
            request_impl! {
                self; $crate::net::Method::Get;
                $($body)+
            }
        }
    );
}

/// Create a service method wrapping a POST request.
///
/// ##Example
/// ```notest
/// service! {
///     pub trait RegisterService {
///         post! {
///             fn register(&self, username: &str, password: &str) -> RegisterResponse {
///                 url = "/register";
///                 fields!{
///                     "username": username,
///                     "password": password
///                 }
///             }
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! post {
    (
        $(#[$meta:meta])*
        fn $fnname:ident $(<$($generics:tt)*>)* (&self $($args:tt)*) -> $ret:ty {
                $($body:tt)+
        }
    ) => (
        $(#[$meta])*
        fn $fnname $(<$($generics)*>)* (&self $($args)*) -> $crate::net::Request<Self, $ret> {
            request_impl! {
                self; $crate::net::Method::Post;
                $($body)+
            }
        }
    );
}

/// Create a service method wrapping a PUT request.
#[macro_export]
macro_rules! put {
    (
        $(#[$meta:meta])*
        fn $fnname:ident $(<$($generics:tt)*>)* (&self $($args:tt)*) -> $ret:ty {
                $($body:tt)+
        }
    ) => (
        $(#[$meta])*
        fn $fnname $(<$($generics)*>)* (&self $($args)*) -> $crate::net::Request<Self, $ret> {
            request_impl! {
                self; $crate::net::Method::Put;
                $($body)+
            }
        }
    );
}

/// Create a service method wrapping a PATCH request.
#[macro_export]
macro_rules! patch {
    (
        $(#[$meta:meta])*
        fn $fnname:ident $(<$($generics:tt)*>)* (&self $($args:tt)*) -> $ret:ty {
                $($body:tt)+
        }
    ) => (
        $(#[$meta])*
        fn $fnname $(<$($generics)*>)* (&self $($args)*) -> $crate::net::Request<Self, $ret> {
            request_impl! {
                self; $crate::net::Method::Patch;
                $($body)+
            }
        }
    );
}

/// Create a service method wrapping a DELETE request.
#[macro_export]
macro_rules! delete {
    (
        $(#[$meta:meta])*
        fn $fnname:ident $(<$($generics:tt)*>)* (&self $($args:tt)*) -> $ret:ty {
                $($body:tt)+
        }
    ) => (
        $(#[$meta])*
        fn $fnname $(<$($generics)*>)* (&self $($args)*) -> $crate::net::Request<Self, $ret> {
            request_impl! {
                self; $crate::net::Method::Delete;
                $($body)+
            }
        }
    );
}