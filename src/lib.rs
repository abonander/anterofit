#![warn(missing_docs)]
#![cfg_attr(feature = "nightly", feature(insert_str))]

#[macro_use]
extern crate quick_error;

extern crate futures;

#[macro_use]
extern crate mime as mime_;

extern crate multipart;
extern crate serde;

extern crate url;

pub extern crate hyper;

mod mime;

pub mod macros;

pub mod net;
pub mod serialize;

pub mod executor;

pub mod error;

pub use error::Error;
pub use error::Never as NeverError;

pub use hyper::Url;

pub use net::Adapter;

pub use net::RawBody;

/// The result type for this crate; used frequently in public APIs.
///
/// Recommended to be used as `anterofit::Result` to avoid confusing
/// shadowing of `std::result::Result`.
pub type Result<T> = ::std::result::Result<T, Error>;





