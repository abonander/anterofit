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
/// use anterofit::{Adapter, Url};
/// use anterofit::executor::DefaultExecutor;
/// use anterofit::net::intercept::NoIntercept;
/// use anterofit::serialize::json::{Serializer as JsonSerializer, Deserializer as JsonDeserializer};
///
/// pub struct DelegateAdapter {
///     // Notice that this field remains private but due to visibility rules,
///     // the impls of `DelegatedService` still get to access it.
///     // This allows you to hide the adapter as an implementation detail.
///     inner: Adapter<DefaultExecutor, NoIntercept, JsonSerializer, JsonDeserializer>,
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

            impl[T: $crate::net::AbsAdapter] for T {
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

            impl[T: $crate::net::AbsAdapter] for T {
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

        impl_impl!($servicenm; [$($guts)*] $($rem)*);
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