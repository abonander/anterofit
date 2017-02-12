//! Wrap REST calls with Rust traits.
//!
//! ```rust
//! #[macro_use] extern crate anterofit;
//! # fn main() {}
//!
//! service! {
//!     /// Trait wrapping `myservice.com` API.
//!     pub trait MyService {
//!         /// Get the version of this API.
//!         fn api_version(&self) -> String {
//!             GET("/version")
//!         }
//!
//!         /// Register a user with the API.
//!         fn register(&self, username: &str, password: &str) {
//!             POST("/register");
//!             fields! {
//!                 username, password
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! # Important Types
//!
//! ## Service Traits
//! Created with the `service!{}` macro as shown above, service traits encompass the actual request
//! submission and response parsing. Each service trait is automatically implemented for
//! `Adapter`, and is object-safe by default, so you can use generic bounds or trait object coercion
//! to narrow the scope:
//!
//! ```rust,ignore
//! fn print_api_version(service: &MyService) {
//!     // This completes synchronously, blocking until the request is complete.
//!     let api_version = service.api_version().exec_here().unwrap();
//!     println!("API version: {}", api_version);
//! }
//!
//! fn register_user<S: MyService>(service: &S, username: &str, password: &str) {
//!     // By default, this will complete asynchronously.
//!     service.register(username, password)
//!         // exec() queues the request on the executor,
//!         // and ignore() silences the `unused_result` lint for `Call`.
//!         .exec().ignore();
//!
//!     // This function returns immediately; all the work is done on the executor.
//! }
//! ```
//!
//! For more details, see the [`service!{}` macro](macro.service.html).
//!
//! ## Adapter
//! Built via `Adapter::builder()`, this is the starting point for all requests. It encompasses
//! five core components, and one very important property:
//!
//! * The `Executor` is responsible for taking prepared requests and executing them. Since Anterofit
//! is primarily designed to be asynchronous, the executor should submit jobs to be completed in the
//! background. Several executors are provided in the `executor` module, but a sane default
//! for low-volume asynchronous requests is provided automatically.
//!
//! * The `Interceptor` is a non-essential but endlessly useful component which can modify
//! request parameters before they are submitted. This currently encompasses modifying the request
//! URL and adding or overwriting HTTP headers. If your app requires some sort of API key or
//! authentication header, you can add an interceptor to your adapter to automatically include
//! the appropriate credentials with each request:
//!
//! ```rust,no_run
//! use anterofit::{Adapter, Url};
//! use anterofit::net::intercept::AddHeader;
//! use anterofit::net::header::{Headers, Authorization, Bearer};
//!
//! let adapter = Adapter::builder()
//!     .base_url(Url::parse("https://myservice.com/api").unwrap())
//!     .interceptor(AddHeader(Authorization (
//!         Bearer {
//!             token: "asdf1234hjkl5678".to_string()
//!         }
//!     )))
//!     .build();
//! ```
//!
//! `Interceptor` is also implemented for closures of the kind `Fn(&mut anterofit::net::request::RequestHead)`,
//! but common operations are implemented as types in the `anterofit::net::intercept` module.
//! You can also chain interceptors together; they will be called in declaration order.
//!
//! * The `Serializer` is responsible for taking a strongly typed request body and converting
//! it to something that can be read into the HTTP stream, such as JSON or a raw byte sequence.
//!
//! * Conversely, the `Deserializer` is responsible for taking a response body in some predetermined
//! format, such as JSON or XML, and reading out a strongly typed value.
//!
//! If you just want JSON serialization and deserialization and don't care about the details,
//! use the `serialize_json()` method of your adapter builder to set the serializer and deserializer
//! simultaneously.
//!
//! * The `Client` (`hyper::client::Client`) is responsible for managing proxies, DNS resolution,
//! and bootstrapping connections. A default instance will be constructed automatically if one is
//! not provided, but you can configure your own instance to tweak some low-level stuff like
//! timeouts or to use a particular proxy.
//!
//! * Finally, the `base_url`, if provided, is automatically prepended to every request URL. This would
//! generally be the protocol, domain and perhaps a path prefix, while request URLs can be standalone paths.
//! That way you can easily swap between, for example, testing and production endpoints implementing
//! the same REST API:
//!
//! ```rust
//! # extern crate anterofit;
//! # fn print_api_version<T>(_: &T) {}
//! # fn register_user<T>(_: &T, _: &str, _: &str) {}
//! # fn main() {
//! use anterofit::{Adapter, Url};
//!
//! let adapter = Adapter::builder()
//!     .base_url(Url::parse("https://test.myservice.com/api").unwrap())
//!     .build();
//!
//! print_api_version(&adapter);
//! register_user(&adapter, "username", "password");
//!
//! let adapter = Adapter::builder()
//!     .base_url(Url::parse("https://prod.myservice.com/api").unwrap())
//!     .build();
//!
//! print_api_version(&adapter);
//! register_user(&adapter, "username", "password");
//! # }
//! ```
//!
//! ## `Request`
//! This type wraps the return value of every service trait method. Unlike in Retrofit,
//! where the request is determined to
//! be synchronous or asynchronous at the service method declaration site^1, `Request` gives the power
//! over this choice to the caller so that no change to the trait is needed to change the execution
//! context:
//!
//! ```rust,ignore
//! fn print_api_version(service: &MyService) {
//!     service.api_version()
//!         // This closure will be called with the `String` value on the executor
//!         .on_complete(|api_version| println!("API version: {}", api_version))
//!         // We don't care about the result since it's `()` anyway.
//!         .exec().ignore();
//! }
//! ```
//! ^1 : Retrofit v1 established synchronicity at the declaration site; v2 follows the same
//! pattern as Anterofit, but the two were developed independently.
//!
//! ## `Call`
//! Returned by `Request::exec()`, this type is a pollable `Future` which will yield the result
//! of the request when it is ready. If there was an error in constructing the request,
//! the result will be available immediately. `Call` provides alternative methods wrapping
//! `Future::poll()` and `Future::wait()` without external types so you
//! have a choice over whether you want to use futures in your app or not.
#![warn(missing_docs)]
#![cfg_attr(feature = "nightly", feature(insert_str, specialization))]
#![recursion_limit="100"]

#[macro_use]
extern crate mime as mime_;

#[macro_use]
extern crate quick_error;

extern crate futures;

extern crate crossbeam;
extern crate parking_lot;

extern crate multipart;

#[cfg(feature = "rustc-serialize")]
extern crate rustc_serialize;

extern crate url;

pub extern crate hyper;

mod adapter;

#[macro_use]
mod macros;

mod mpmc;

pub mod mime;

pub mod net;

pub mod serialize;

pub mod executor;

pub mod error;

pub use error::Error;

pub use hyper::Url;

pub use adapter::{Adapter, AbsAdapter, AdapterBuilder, ObjSafeAdapter, InterceptorMut};

#[cfg(any(feature = "rustc-serialize", feature = "serde-json"))]
pub use adapter::JsonAdapter;

pub use net::body::RawBody;

pub use net::request::Request;

/// The result type for this crate; used frequently in public APIs.
///
/// Recommended to be used as `anterofit::Result` to avoid confusing
/// shadowing of `std::result::Result`.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Strong typing hint for delegate adapter-getters.
#[doc(hidden)]
pub fn get_adapter<D, A: AbsAdapter, F: FnOnce(&D) -> &A>(delegate: &D, map: F) -> &A {
    map(delegate)
}