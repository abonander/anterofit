// FIXME before release
//#![warn(missing_docs)]
#![feature(insert_str)]

#[macro_use]
extern crate quick_error;

extern crate futures;

#[macro_use]
#[macro_reexport]
extern crate mime as mime_;

extern crate multipart;
extern crate serde;

extern crate url;

pub extern crate hyper;

mod mime;

#[macro_export]
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

pub type Result<T> = ::std::result::Result<T, Error>;





