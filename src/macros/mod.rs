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
///
/// ##Generics and `where` clauses
/// Both of these are supported; however, the Rust grammar must be changed slightly
/// so that they can be parsed and transformed properly by the `service!{}` macro.
///
/// Put simply, use `[ ]` instead of `< >` to wrap your generic declarations,
/// and wrap your entire `where` clause, if present, with `[ ]`:
///
/// ```rust
/// # #[macro_use] extern crate anterofit;
/// # fn main() {}
/// service! {
///     pub trait MyService {
///         /// Register a new user with the API.
///         fn register[U: ToString, P: ToString](&self, username: U, password: P) {
///             POST("/register");
///             fields! {
///                 username, password
///             }
///         }
///
///         /// Login an existing user with the API.
///         fn login[U, P](&self, username: U, password: P) [where U: ToString, P: ToString] {
///             POST("/login");
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

            delegate[T: $crate::net::AbsAdapter] for T {
                |this| this
            }
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

            delegate[T: $crate::net::AbsAdapter] for T {
                |this| this
            }
        }
    );
    (
        $(#[$meta:meta])*
        trait $servicenm:ident {
            $($guts:tt)*
        }

        $($delegates:tt)+
    ) => (
        $(#[$meta])*
        trait $servicenm {
            method_proto!($($guts)*);
        }

        delegate_impl!($servicenm; [$($guts)*] $($delegates)+);
    );
    (
        $(#[$meta:meta])*
        pub trait $servicenm:ident {
            $($guts:tt)*
        }

        $($delegates:tt)+
    ) => (
        $(#[$meta])*
        pub trait $servicenm {
            method_proto!($($guts)*);
        }

        delegate_impl!($servicenm; [$($guts)*] $($delegates)+);
    );
}

#[doc(hidden)]
#[macro_export]
macro_rules! method_proto(
    // Plain declaration
    (
        $(#[$fnmeta:meta])*
        fn $fnname:ident (&self $($args:tt)*) $(-> $ret:ty)* {
            $($body:tt)+
        }
        
        $($rem:tt)*
    ) => (
        $(#[$fnmeta])*
        fn $fnname (&self $($args)*)  -> $crate::net::Request<$($ret)*>;
        
        method_proto!($($rem)*);
    );
    // Generics
    (
        $(#[$fnmeta:meta])*
        fn $fnname:ident [$($generics:tt)+] (&self $($args:tt)*) $(-> $ret:ty)* {
            $($body:tt)+
        }
        
        $($rem:tt)*
    ) => (
        $(#[$fnmeta])*
        fn $fnname <$($generics)+> (&self $($args)*) -> $crate::net::Request<$($ret)*>;
        
        method_proto!($($rem)*);
    );
    // Where clause
    (
        $(#[$fnmeta:meta])*
        fn $fnname:ident  (&self $($args:tt)*) $(-> $ret:ty)* [where $($wheres:tt)+] {
            $($body:tt)+
        }
        
        $($rem:tt)*
    ) => (
        $(#[$fnmeta])*
        fn $fnname (&self $($args)*) -> $crate::net::Request<$($ret)*> where $($wheres)+ ;
        
        method_proto!($($rem)*);
    );
    // Generics + where clause
    (
        $(#[$fnmeta:meta])*
        fn $fnname:ident [$($generics:tt)+] (&self $($args:tt)*) $(-> $ret:ty)* [where $($wheres:tt)+] {
            $($body:tt)+
        }
        
        $($rem:tt)*
    ) => (
        $(#[$fnmeta])*
        fn $fnname <$($generics)+> (&self $($args)*) -> $crate::net::Request<$($ret)*> where $($wheres)+;
        
        method_proto!($($rem)*);
    );
    // Empty end case for recursion
    () => ();
);

#[doc(hidden)]
#[macro_export]
macro_rules! method_impl(
    // Plain declaration
    (
        $getadapt:expr;

        $(#[$fnmeta:meta])*
        fn $fnname:ident (&self $($args:tt)*) $(-> $ret:ty)* {
            $($body:tt)+
        }
        
        $($rem:tt)*
    ) => (
        $(#[$fnmeta])*
        fn $fnname (&self $($args)*)  -> $crate::net::Request<$($ret)*> {
            request_impl! {
                $crate::get_adapter(self, $getadapt); $($body)+
            }
        }
        
        method_impl!($getadapt; $($rem)*);
    );
    // Generics
    (
        $getadapt:expr;

        $(#[$fnmeta:meta])*
        fn $fnname:ident [$($generics:tt)+] (&self $($args:tt)*) $(-> $ret:ty)* {
            $($body:tt)+
        }
        
        $($rem:tt)*
    ) => (
        $(#[$fnmeta])*
        fn $fnname <$($generics)+> (&self $($args)*) -> $crate::net::Request<$($ret)*> {
            request_impl! {
                $crate::get_adapter(self, $getadapt); $($body)+
            }
        }
        
        method_impl!($getadapt; $($rem)*);
    );
    // Where clause
    (
        $getadapt:expr;

        $(#[$fnmeta:meta])*
        fn $fnname:ident  (&self $($args:tt)*) $(-> $ret:ty)* [where $($wheres:tt)+] {
            $($body:tt)+
        }
        
        $($rem:tt)*
    ) => (
        $(#[$fnmeta])*
        fn $fnname (&self $($args)*) -> $crate::net::Request<$($ret)*> where $($wheres)+ {
            request_impl! {
                $crate::get_adapter(self, $getadapt); $($body)+
            }
        }
        
        method_impl!($getadapt; $($rem)*);
    );
    // Generics + Where clause
    (
        $getadapt:expr;

        $(#[$fnmeta:meta])*
        fn $fnname:ident [$($generics:tt)+] (&self $($args:tt)*) $(-> $ret:ty)* [where $($wheres:tt)+] {
            $($body:tt)+
        }
        
        $($rem:tt)*
    ) => (
        $(#[$fnmeta])*
        fn $fnname <$($generics)+> (&self $($args)*) -> $crate::net::Request<$($ret)*> where $($wheres)+ {
            request_impl! {
                $crate::get_adapter(self, $getadapt); $($body)+
            }
        }
        
        method_impl!($getadapt; $($rem)*);
    );
    // Empty end-case for recursion
    ($_getadapt:expr; ) => ();
);

#[doc(hidden)]
#[macro_export]
macro_rules! delegate_impl {
    (
        $servicenm:ident; [$($guts:tt)*]
        delegate for $delegate:path {
            $getadpt:expr
        }

        $($rem:tt)*
    ) => (
        impl $servicenm for $delegate {
            method_impl!($getadapt; $($guts)*);
        }

        delegate_impl!($servicenm; [$($guts)*] $($rem)*);
    );
    (
        $servicenm:ident; [$($guts:tt)*]
        delegate [$($decls:tt)*] for $delegate:path {
            $getadapt:expr
        }

        $($rem:tt)*
    ) => (
        impl<$($decls)*> $servicenm for $delegate {
            method_impl!($getadapt; $($guts)*);
        }

        delegate_impl!($servicenm; [$($guts)*] $($rem)*);
    );
    (
        $servicenm:ident; [$($guts:tt)*]
        delegate for $delegate:path [where $($wheres:tt)+]{
            $getadapt:expr
        }

        $($rem:tt)*
    ) => (
        impl $servicenm for $delegate where $($wheres:tt)+ {
            method_impl!($getadapt; $($guts)*);
        }

        delegate_impl!($servicenm; [$($guts)*] $($rem)*);
    );
    (
        $servicenm:ident; [$($guts:tt)*]
        delegate [$($decls:tt)*] for $delegate:path [where $($wheres:tt)+]{
            $getadapt:expr
        }

        $($rem:tt)*
    ) => (
        impl<$($decls)*> $servicenm for $delegate where $($wheres:tt)+ {
            method_impl!($getadapt; $($guts)*);
        }

        delegate_impl!($servicenm; [$($guts)*] $($rem)*);
    );
    // Empty end-case for recursion
    ($servicenm:ident; [$($guts:tt)*]) => ();
}