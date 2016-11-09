// FIXME before release
//#![warn(missing_docs)]

#[macro_use]
extern crate quick_error;

extern crate futures;

extern crate hyper;
extern crate mime;
extern crate multipart;
extern crate serde;

extern crate url;

#[macro_export]
pub mod macros;
pub mod net;
pub mod serialize;

pub mod executor;

pub mod error;

pub use error::Error;
pub use error::Never as NeverError;

pub type Result<T> = Result<T, Error>;




