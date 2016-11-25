//! Macros for Anterofit.
#[macro_use]
mod method;
#[macro_use]
mod request;

/// Define a service trait whose methods make HTTP requests.
///
/// ##Example
/// ```notest
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
            $($methods:tt)*
        }
    ) => (
        pub trait $servicenm : $crate::net::RequestAdapter {
            $($methods)*
        }

        impl<T: $crate::net::RequestAdapter> $servicenm for T {}
    )
}