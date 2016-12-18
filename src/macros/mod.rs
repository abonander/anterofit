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
            $($guts:tt)*
        }
    ) => (
        service! {
            $(#[$meta])*
            trait $servicenm {
                $($guts)*
            }

            delegate(T: $crate::net::AbsAdapter) for T {
                |this| this
            }
        }
    );
    (
        $(#[$meta:meta])*
        trait $servicenm:ident {
            $(
                $fnitem:item
            )*
        }

        delegate($($delegatedecls:tt)*) for $delegate:ty
        $(where $delwherety:ty : $delwherebnd:ty)* $(, $ndelwherety:ty : $ndelwherebnd:ty)* {
            $getadapter:expr
        }
    ) => (
        $(#[$meta])*
        trait $servicenm {
            $(
                method_proto!($fnitem);
            )*
        }

        impl<$($delegatedecls)*> $servicenm for $delegate
        $(where $delwherety : $delwherebnd)* $(, $ndelwherety : $ndelwherebnd)* {
            $(
                method_impl!($getadapter; $fnitem);
            )*
        }
    );
    (
        $(#[$meta:meta])*
        pub trait $servicenm:ident {
            $($guts:tt)*
        }
    ) => (
        service! {
            $(#[$meta])*
            pub trait $servicenm {
                $($guts)*
            }

            delegate(T: $crate::net::AbsAdapter) for T {
                |this| this
            }
        }
    );
    (
        $(#[$meta:meta])*
        pub trait $servicenm:ident {
            $(
                $fnitem:item
            )*
        }

        delegate($($delegatedecls:tt)*) for $delegate:ty
        $(where $delwherety:ty : $delwherebnd:ty)* $(, $ndelwherety:ty : $ndelwherebnd:ty)* {
            $getadapter:expr
        }
    ) => (
        $(#[$meta])*
        pub trait $servicenm {
            $(
                method_proto!($fnitem);
            )*
        }

        impl<$($delegatedecls)*> $servicenm for $delegate
        $(where $delwherety : $delwherebnd)* $(, $ndelwherety : $ndelwherebnd)* {
            $(
                method_impl!($getadapter; $fnitem);
            )*
        }
    );
}

#[doc(hidden)]
#[macro_export]
macro_rules! method_proto {
    (
        $(#[$fnmeta:meta])*
        fn $fnname ($($args:tt)*) $(-> $ret:ty)* {
            // Remainder tokens that aren't necessary for prototype
            $($_rem:tt)*
        }
    ) => (
        $(#[$fnmeta])*
        fn $fnname (&self $($args)*) -> $crate::net::Request<$($ret)*>;
    );
    (
        $(#[$fnmeta:meta])*
        fn $fnname <$($genericty:ident $(: $genericbnd:ty)*),*> ($($args:tt)*) $(-> $ret:ty)* {
            // Remainder tokens that aren't necessary for prototype
            $($_rem:tt)*
        }
    ) => (
        $(#[$fnmeta])*
        fn $fnname <$($genericty $(: $genericbnd)*),*> (&self $($args)*) -> $crate::net::Request<$($ret)*>;
    );
    (
        $(#[$fnmeta:meta])*
        fn $fnname ($($args:tt)*) $(-> $ret:ty)*
        where $($wherety:path : $wherebnd:path),+ {
            // Remainder tokens that aren't necessary for prototype
            $($_rem:tt)*
        }
    ) => (
        $(#[$fnmeta])*
        fn $fnname (&self $($args)*) -> $crate::net::Request<$($ret)*>
        where $($wherety : $wherebnd),+;
    );
    (
        $(#[$fnmeta:meta])*
        fn $fnname <$($genericty:ident $(: $genericbnd:ty)*),*> ($($args:tt)*) $(-> $ret:ty)*
        where $($wherety:path : $wherebnd:path),+ {
            // Remainder tokens that aren't necessary for prototype
            $($_rem:tt)*
        }
    ) => (
        $(#[$fnmeta])*
        fn $fnname <$($genericty $(: $genericbnd)*),*> (&self $($args)*) -> $crate::net::Request<$($ret)*>
        where $($wherety : $wherebnd),+;
    );
}

#[doc(hidden)]
#[macro_export]
macro_rules! method_impl {
    (
        $getadapter:expr;
        $(#[$fnmeta:meta])*
        fn $fnname (&self $($args:tt)*) $(-> $ret:ty)* {
            $($body:tt)*
        }
    ) => (
        fn $fnname (&self $($args)*) -> $crate::net::Request<$($ret)*> {
            request_impl! {
                $crate::get_adapter(self, $getadapter); $($body)+
            }
        }
    );
    (
        $getadapter:expr;
        $(#[$fnmeta:meta])*
        fn $fnname <$($genericty:ident $(: $genericbnd:ty)*),*> (&self $($args:tt)*) $(-> $ret:ty)* {
            $($body:tt)*
        }
    ) => (
        fn $fnname <$($genericty $(: $genericbnd)*),*> (&self $($args)*) -> $crate::net::Request<$($ret)*> {
            request_impl! {
                $crate::get_adapter(self, $getadapter); $($body)+
            }
        }
    );
    (
        $getadapter:expr;
        $(#[$fnmeta:meta])*
        fn $fnname (&self $($args:tt)*) $(-> $ret:ty)*
        where $($wherety:path : $wherebnd:path),+ {
            $($body:tt)*
        }
    ) => (
        fn $fnname (&self $($args)*) -> $crate::net::Request<$($ret)*>
        where $($wherety : $wherebnd),+ {
            request_impl! {
                $crate::get_adapter(self, $getadapter); $($body)+
            }
        }
    );
    (
        $getadapter:expr;
        $(#[$fnmeta:meta])*
        fn $fnname <$($genericty:ident $(: $genericbnd:ty)*),*> (&self $($args:tt)*) $(-> $ret:ty)*
        where $($wherety:path : $wherebnd:path),+ {
            $($body:tt)*
        }
    ) => (
        fn $fnname <$($genericty $(: $genericbnd)*),*> (&self $($args)*) -> $crate::net::Request<$($ret)*>
        where $($wherety : $wherebnd),+ {
            request_impl! {
                $crate::get_adapter(self, $getadapter); $($body)+
            }
        }
    );
}