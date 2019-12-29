//! Anterofit's HTTP client framework, built on Hyper.
//!
//! Works standalone, but designed to be used with the `service!{}` macro.

pub use hyper::method::Method;

pub use hyper::header::Headers;

pub use hyper::header;

pub use self::intercept::{Chain, Interceptor};

pub use self::call::Call;

pub use self::request::{Request, RequestBuilder, RequestHead};

pub use self::response::{FromResponse, Raw as RawResponse};

pub mod body;

mod call;

pub mod intercept;

pub mod method;

pub mod request;

pub mod response;
