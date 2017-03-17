//! Macros for Anterofit.

#[macro_use]
mod request;

/// Define a service trait whose methods make HTTP requests.
///
/// ##Example
/// ```rust
/// # #[macro_use] extern crate anterofit;
/// # fn main() {}
/// pub type ApiToken = String;
///
/// service! {
///     pub trait MyService {
///         /// Get the version of this API.
///         fn api_version(&self) -> String {
///             GET("/version")
///         }
///
///         /// Register a new user with the API.
///         fn register(&self, username: &str, password: &str) {
///             POST("/register");
///             fields! {
///                 username, password
///             }
///         }
///
///         /// Login an existing user with the API, returning the API token.
///         fn login(&self, username: &str, password: &str) -> ApiToken {
///             POST("/login");
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
/// so that they can be parsed and transformed properly by the `service!{}` macro without
/// making its implementation details too complex.
///
/// Put simply, use `[]` instead of `<>` to wrap your generic declarations,
/// and wrap your entire `where` clause, if present, with `[]`:
///
/// ```rust
/// # #[macro_use] extern crate anterofit;
/// # fn main() {}
/// pub type ApiToken = String;
///
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
///         fn login[U, P](&self, username: U, password: P) -> ApiToken
///         [where U: ToString, P: ToString] {
///             POST("/login");
///             fields! {
///                 username, password
///             }
///         }
///     }
/// }
/// ```
///
/// ##Delegates
/// By default, every service trait declared with `service!{}` has a blanket-impl for
/// `T: anterofit::AbsAdapter`, which makes it most useful for the default use-case, where you're
/// using Anterofit to make HTTP requests within your application.
///
/// However, if you want to use Anterofit to create a library wrapping some REST API, such as [Github's](https://developer.github.com/v3/),
/// this blanket impl is not so useful as you will probably want to create your own wrapper for Anterofit's
/// adapter that always uses the correct base URL, serializer/deserializer, adds auth tokens, etc.
///
/// In this case, you can declare one or more delegate impls which will be used instead of the default
/// blanket impl; the only requirement of these delegate impl declarations is that they provide an
/// accessor for an underlying `AbsAdapter` implementation (which is only required to be
/// visible to the declaring module, allowing an opaque abstraction while using service traits
/// in a public API). The accessor is an expression that resolves to an `FnOnce()` closure
/// which is passed the `self` parameter, and is expected to return `&T` where `T: AbsAdapter`.
///
/// ```rust
/// # #[macro_use] extern crate anterofit;
/// # // This mess of cfg's is required to make sure this is a no-op when the `serde` feature is enabled.
/// # #[cfg(feature = "rustc-serialize")]
/// extern crate rustc_serialize;
/// # fn main() {}
/// # #[cfg(feature = "rustc-serialize")]
/// # mod only_rustc_serialize {
///
/// use anterofit::{Adapter, JsonAdapter, Url};
///
/// pub struct DelegateAdapter {
///     // Notice that this field remains private but due to visibility rules,
///     // the impls of `DelegatedService` still get to access it.
///     // This allows you to hide the adapter as an implementation detail.
///     inner: JsonAdapter
/// }
///
/// impl DelegateAdapter {
///     pub fn new() -> Self {
///         let adapter = Adapter::builder()
///             .serialize_json()
///             .base_url(Url::parse("https://myservice.com").unwrap())
///             .build();
///
///         DelegateAdapter {
///             inner: adapter,
///         }
///     }
/// }
///
/// // If using the `serde` feature, you would use `#[derive(Deserialize)]` instead
/// // and `extern crate serde;` at the crate root.
/// #[derive(RustcDecodable)]
/// pub struct Record {
///     pub id: u64,
///     pub title: String,
///     pub body: String,
/// }
///
/// service! {
///     pub trait DelegatedService {
///         /// Create a new record, returning the record ID.
///         fn create_record(&self, title: &str, body: &str) -> u64 {
///             POST("/record");
///             fields! { title, body }
///         }
///
///         /// Get an existing record by ID.
///         fn get_record(&self, record_id: u64) -> Record {
///             GET("/record/{}", record_id)
///         }
///     }
///
///     // This generates `impl DelegatedService for DelegateAdapter {}`
///     impl for DelegateAdapter {
///         // Closure parameter is just `&self` from the service method body.
///         |this| &this.inner
///     }
///
///     // Generics and `where` clauses are allowed in their usual positions, however `[]` is
///     // required in the same places as mentioned under the previous header.
///     impl[T] for T [where T: AsRef<DelegateAdapter>] {
///         |this| &this.as_ref().inner
///     }
///
///     // As shown here, multiple declarations are allowed as well.
/// }
/// # }
/// ```
#[cfg(not(feature = "service-attr"))]
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

            impl[T: $crate::AbsAdapter] for T {
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

            impl[T: $crate::AbsAdapter] for T {
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

/// Create an implementation of `UnsizeService` for the given service trait.
///
/// Note that this only works with object-safe traits, and does *not* work with
/// traits that have delegate impls. Can be invoked with more than one name at a time.
#[macro_export]
macro_rules! unsizeable(
    ($($servicenm:ty),+) => (
        $(impl ::anterofit::UnsizeService for $servicenm {
            fn from_adapter<A>(adpt: ::std::sync::Arc<A>) -> ::std::sync::Arc<Self>
            where A: ::anterofit::AbsAdapter {
                adpt
            }
        })*
    )
);

#[doc(hidden)]
#[macro_export]
macro_rules! method_proto (
    ($($body:tt)*) => (
        parse_fn_decl!(without_block!(), $($body)*);
    )
);

#[doc(hidden)]
#[macro_export]
macro_rules! method_impl (
    ($getadpt:expr; $($body:tt)*) => (
        parse_fn_decl!(with_block!($getadpt; ), $($body)*);
    )
);

#[doc(hidden)]
#[macro_export]
macro_rules! without_block (
    // Plain declaration
    (
        [$($proto:tt)+][(&self $($args:tt)*) -> $ret:ty][$($clause:tt)*][$blk:block]
    ) => (
        $($proto)+ (&self $($args)*) -> $ret $($clause)*;
    );
);

#[doc(hidden)]
#[macro_export]
macro_rules! with_block (
    // Plain declaration
    (
        $getadapt:expr; [$($proto:tt)+][(&self $($args:tt)*) -> $ret:ty][$($clause:tt)*][ { $($body:tt)+ } ]
    ) => (
        $($proto)+ (&self $($args)*) -> $ret $($clause)* {
            request_impl! {
                $crate::get_adapter(self, $getadapt); $($body)+
            }
        }
    );
);

#[doc(hidden)]
#[macro_export]
macro_rules! parse_fn_decl (
    (@emit [$($b4gen:tt)+][$($constr:tt)*] $cb:ident ! ($($cbargs:tt)*), [$($sig:tt)+]{ clause: [$($clause:tt)*]}, {$($body:tt)+} $($rest:tt)*) => (
        $cb!($($cbargs)* [$($b4gen)+ $($constr)*][$($sig)+][$($clause)*] [{ $($body)+ }]);
        parse_fn_decl!($cb! ($($cbargs)*), $($rest)*);
    );
    (@transform [$($b4gen:tt)+] $cb:ident ! ($($cbargs:tt)*), { constr: [$($constr:tt)*], $($other:tt)*}, $($rest:tt)+) => (
        transform_sig! {
            (@emit [$($b4gen)+][$($constr)*] $cb ! ($($cbargs)*), )
            $($rest)+
        }
    );
    (@generics [$($b4gen:tt)+] $cb:ident ! ($($cbargs:tt)*), [$($rest:tt)+]) => (
        parse_generics_shim! {
            { constr },
            then parse_fn_decl ! (@transform [$($b4gen)+] $cb ! ($($cbargs)*), ),
            $($rest)+
        }
    );
    ($cb:ident ! ($($cbargs:tt)*), $(#[$meta:meta])* fn $fnname:ident $($rest:tt)+) => (
        parse_fn_decl!(@generics [$(#[$meta])* fn $fnname] $cb ! ($($cbargs)*), [$($rest)+]);
    );
    ($cb:ident ! ($($cbargs:tt)*), $(#[$meta:meta])* pub fn $fnname:ident $($rest:tt)+) => (
        parse_fn_decl!(@generics [$(#[$meta])* pub fn $fnname] $cb ! ($($cbargs)*), [$($rest)+]);
    );
    ($cb:ident ! ($($cbargs:tt)*), ) => ()
);

#[doc(hidden)]
#[macro_export]
macro_rules! transform_sig (
    // Force return type
    (($($preargs:tt)*) (&self $($args:tt)*) -!> $ret:ty {$($body:tt)*} $($rest:tt)* ) => (
        parse_fn_decl!($($preargs)* [(&self $($args)*) -> $ret] { clause: []}, {$($body)*} $($rest)*)
    );
    (($($preargs:tt)*) (&self $($args:tt)*) -!> $ret:ty where $($rest:tt)+ ) => (
        parse_where_shim! {
            { clause },
            then parse_fn_decl!($($preargs)* [(&self $($args)*) -> $ret] ),
            where $($rest)+
        }
    );
    (($($preargs:tt)*) (&self $($args:tt)*) -> $ret:ty {$($body:tt)*} $($rest:tt)*) => (
        parse_fn_decl!($($preargs)* [(&self $($args)*) -> $crate::Request<$ret>]
                       { clause: []}, {$($body)*} $($rest)*);
    );
    (($($preargs:tt)*) (&self $($args:tt)*) -> $ret:ty where $($rest:tt)+ ) => (
        parse_where_shim! {
            { clause },
            then parse_fn_decl!($($preargs)* [(&self $($args)*) -> $crate::Request<$ret>] ),
            where $($rest)+
        }
    );
    (($($preargs:tt)*) (&self $($args:tt)*) {$($body:tt)*} $($rest:tt)*) => (
        parse_fn_decl!($($preargs)* [(&self $($args)*) -> $crate::Request]
                       { clause: []}, {$($body)*} $($rest)*);
    );
    (($($preargs:tt)*) (&self $($args:tt)*) where $($rest:tt)+ ) => (
        parse_where_shim! {
            { clause },
            then parse_fn_decl!($($preargs)* [(&self $($args)*) -> $crate::Request] ),
            where $($rest)+
        }
    );
);

#[doc(hidden)]
#[macro_export]
macro_rules! delegate_impl {
    (
        $servicenm:ident; [$($guts:tt)*]
        impl for $delegate:ty {
            $getadapt:expr
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
        impl [$($decls:tt)*] for $delegate:ty {
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
        impl for $delegate:ty [where $($wheres:tt)+]{
            $getadapt:expr
        }

        $($rem:tt)*
    ) => (
        impl $servicenm for $delegate where $($wheres)+ {
            method_impl!($getadapt; $($guts)*);
        }

        delegate_impl!($servicenm; [$($guts)*] $($rem)*);
    );
    (
        $servicenm:ident; [$($guts:tt)*]
        impl [$($decls:tt)*] for $delegate:ty [where $($wheres:tt)+]{
            $getadapt:expr
        }

        $($rem:tt)*
    ) => (
        impl<$($decls)*> $servicenm for $delegate where $($wheres)+ {
            method_impl!($getadapt; $($guts)*);
        }

        delegate_impl!($servicenm; [$($guts)*] $($rem)*);
    );
    // Empty end-case for recursion
    ($servicenm:ident; [$($guts:tt)*]) => ();
}

/// Create a meta-service trait which combines the listed service traits.
///
/// This can be used as a shorthand to combine several service traits in generics
/// or trait objects, and then upcast at-will:
///
/// ```no_run
/// #[macro_use] extern crate anterofit;
///
/// use anterofit::Adapter;
///
/// service! {
///     trait FooService {
///         fn foo(&self) -> String {
///             GET("/foo")
///         }
///     }
/// }
///
/// service! {
///     trait BarService {
///         fn bar(&self, param: &str) {
///             POST("/bar");
///             query! { "param" => param }
///         }
///     }
/// }
///
/// meta_service! { trait BazService: FooService + BarService }
///
/// fn use_baz<T: BazService>(service: &T) {
///     service.foo().exec_here().unwrap();
///     service.bar("Hello, world!").exec_here().unwrap();
/// }
///
/// fn obj_baz(service: &BazService) {
///     service.foo().exec_here().unwrap();
///     service.bar("Hello, world!").exec_here().unwrap();
/// }
///
/// # fn main() {
/// // Useless adapter, just for demonstration
/// let adapter = Adapter::builder().build();
///
/// use_baz(&adapter);
/// obj_baz(&adapter);
/// # }
/// ```
#[macro_export]
macro_rules! meta_service (
    (trait $metanm:ident : $($superr:tt)+ ) => (
        trait $metanm : $($superr)+ {}

        impl<T: $($superr)+> $metanm for T {}
    );

    (pub trait $metanm:ident : $($superr:tt)+ ) => (
        pub trait $metanm : $($superr)+ {}

        impl<T: $($superr)+> $metanm for T {}
    );
);

// No-op macro to silence errors in IDEs when using `#[service]`
#[cfg(feature = "service-attr")]
#[doc(hidden)]
#[macro_export]
macro_rules! delegate (
    ($($a:tt)*) => ()
);
