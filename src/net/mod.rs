//! Anterofit's HTTP client framework, built on Hyper.
//!
//! Works standalone, but designed to be used with the `service!{}` macro.

pub use hyper::method::Method;

pub use hyper::header::Headers;

pub use hyper::header;

pub use self::adapter::{Adapter, AbsAdapter, AdapterBuilder, ObjSafeAdapter, InterceptorMut};

#[cfg(any(feature = "rustc-serialize", feature = "serde-json"))]
pub use self::adapter::JsonAdapter;

pub use self::intercept::{Interceptor, Chain};

pub use self::call::Call;

pub use self::request::{RequestHead, RequestBuilder, Request};

pub use self::response::{FromResponse, Raw as RawResponse};

mod adapter;

pub mod body;

mod call;

pub mod intercept;

pub mod request;

pub mod response;