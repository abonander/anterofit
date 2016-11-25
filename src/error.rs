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

use serialize::none::NoSerializeError;

use std::io::Error as IoError;
use std::error::Error as StdError;

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
        /// Errors that occur during serialization.
        Serialize(e: Box<StdError + Send + 'static>) {
            cause(&**e)
            description(e.description())
        }

        /// Errors that occur during deserialization.
        Deserialize(e: Box<StdError + Send + 'static>) {
            cause(&**e)
            description(e.description())
        }
        /// The `std::io::Error` type.
        ///
        /// Associated with miscellaneous errors dealing with I/O streams.
        StdIo(e: IoError){
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
        /// Returned when a service method requests (de)serialization, but no (de)serializer was provided.
        ///
        /// Check the error description for which.
        NoSerialize(e: NoSerializeError) {
            from()
            cause(e)
            description(e.description())
        }
        /// The miscellaneous error type, can be anything.
        Other(e: Box<StdError + Send + 'static>){
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
    }
}

impl Error {
    /// Map the result, boxing and wrapping the error as `Error::Serialize`
    pub fn map_serialize<T, E: StdError + Send + 'static>(res: Result<T, E>) -> Result<T, Self> {
        res.map_err(|e| Error::Serialize(Box::new(e)))
    }

    /// Map the result, boxing and wrapping the error as `Error::Deserialize`
    pub fn map_deserialize<T, E: StdError + Send + 'static>(res: Result<T, E>) -> Result<T, Self> {
        res.map_err(|e| Error::Deserialize(Box::new(e)))
    }
}

/// Flatten a `Result` of a `Result` where the outer's error type is convertible to `anterofit::Result`.
pub fn flatten_res<T, E>(res: Result<Result<T, Error>, E>) -> Result<T, Error> where Error: From<E> {
    try!(res)
}