//! Wrap REST calls with Rust traits.
//!
//! ```rust,ignore
//! service! {
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

#![warn(missing_docs)]
#![cfg_attr(feature = "nightly", feature(insert_str))]
#![recursion_limit="100"]

#[macro_use]
extern crate quick_error;

extern crate futures;

#[macro_use]
extern crate mime as mime_;

extern crate multipart;

#[cfg(feature = "rustc-serialize")]
extern crate rustc_serialize;

extern crate url;

pub extern crate hyper;

#[macro_use]
mod macros;

pub mod mime;

pub mod net;

pub mod serialize;

pub mod executor;

pub mod error;

pub use error::Error;

pub use hyper::Url;

pub use net::{Adapter, AbsAdapter};

pub use net::body::RawBody;

pub use net::request::Request;

/// The result type for this crate; used frequently in public APIs.
///
/// Recommended to be used as `anterofit::Result` to avoid confusing
/// shadowing of `std::result::Result`.
pub type Result<T> = ::std::result::Result<T, Error>;