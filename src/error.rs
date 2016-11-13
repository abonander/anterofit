//! Assorted error types and helper functions used by this crate.

/// Error type from the `hyper` crate.
///
/// Associated with errors from connection issues or I/O issues with sockets.
pub use hyper::Error as HyperError;

/// Error type from the `url` crate.
///
/// Associated with errors with URL string parsing or concatenation.
pub use hyper::error::ParseError as UrlError;

/// Error type from the `multipart` crate.
///
/// Associated with errors writing out `multipart/form-data` requests.
pub type MultipartError = ::multipart::client::lazy::LazyIoError<'static>;

use serialize::{NoSerializerError, NoDeserializerError};

use std::io::Error as IoError;
use std::error::Error as StdError;
use std::fmt;

quick_error! {
    /// The error type for this crate.
    ///
    /// Can be converted from basically any error returned by any crate used here.
    #[derive(Debug)]
    pub enum Error {
        /// Error type from the `hyper` crate.
        ///
        /// Associated with errors from connection issues or I/O issues with sockets.
        Hyper(e: HyperError) {
            from()
            cause(e)
            description(e.description())
        }
        /// Error type from the `url` crate.
        ///
        /// Associated with errors with URL string parsing or concatenation.
        Url(e: UrlError) {
            from()
            cause(e)
            description(e.description())
        }
        /// Error type from the `serde_json` crate.
        ///
        /// Associated with JSON (de)serialization errors.
        Json(e: ::serialize::json::Error) {
            from()
            cause(e)
            description(e.description())
        }
        /// Error type from the `serde_xml` crate.
        ///
        /// Associated with XML (de)serialization errors.
        Xml(e: ::serialize::xml::Error) {
            from()
            cause(e)
            description(e.description())
        }
        /// The `std::io::Error` type.
        ///
        /// Associated with miscellaneous errors dealing with I/O streams.
        Io(e: IoError){
            from()
            cause(e)
            description(e.description())
        }
        /// Error type from the `multipart` crate.
        ///
        /// Associated with errors writing out `multipart/form-data` requests.
        Multipart(e: MultipartError) {
            from()
            cause(e)
            description(e.description())
        }
        /// The no-serializer error type.
        ///
        /// Returned when a service method requests serialization, but no serializer was provided.
        NoSerializer(e: NoSerializerError) {
            from()
            cause(e)
            description(e.description())
        }
        /// The no-deserializer error type.
        ///
        /// Returned when a service method requests deserialization, but no deserializer was provided.
        NoDeserializer(e: NoDeserializerError) {
            from()
            cause(e)
            description(e.description())
        }
        /// The miscellaneous error type, can be anything.
        Other(e: Box<StdError + Send>){
            from()
            cause(&**e)
            description(e.description())
        }
        /// The `futures::Canceled` error type.
        ///
        /// Associated with panics on worker threads.
        Panic {
            from(::futures::Canceled)
            description("The request could not be completed because a panic occurred on the worker thread.")
        }
        /// Returned by methods on `Call` if the result was already taken.
        ResultTaken {
            description("The result has already been taken from this Call.")
        }
        /// This error type should never occur. This is only present to satisfy the type-checker.
        __Never(e: Never) {
            from()
            cause(e)
            description(e.description())
        }
    }
}

macro_rules! never (
    ($self_:expr) => (
        unreachable!(
        "Method called on `anterofit::error::Never`, which simply shouldn't be possible.
        Sounds like you probably screwed up somewhere with `unsafe`. `&self`: {:p}", $self_
        );
    )
);

/// An error type which cannot be instantiated.
///
/// Used in methods which are bound by API contract to return `Result` but which are actually infallible.
pub enum Never {}

impl StdError for Never {
    fn description(&self) -> &str {
        never!(self);
    }

    fn cause(&self) -> Option<&StdError> {
        never!(self);
    }
}

impl fmt::Debug for Never {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        never!(self);
    }
}

impl fmt::Display for Never {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        never!(self);
    }
}

/// Flatten a `Result` of a `Result` where the outer's error type is convertible to `anterofit::Result`.
pub fn flatten_res<T, E>(res: Result<Result<T, Error>, E>) -> Result<T, Error> where Error: From<E> {
    try!(res)
}

/// Map a `Result` whose error type is convertible to `anterofit::error::Error`, to `anterofit::Result`.
pub fn map_res<T, E>(res: Result<T, E>) -> Result<T, Error> where Error: From<E> {
    res.map_err(|e| e.into())
}