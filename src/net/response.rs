//! Types concerning the responses from REST calls.

pub use hyper::client::Response;

use std::io::{self, Read};

use serialize::{Deserialize, Deserializer};

use super::adapter::AbsAdapter;

use ::Result;

/// A trait describing types which can be converted from raw response bodies.
///
/// Implemented for `T: Deserialize + Send + 'static`.
///
/// Use `RawResponse` if you just want the response body.
pub trait FromResponse: Send + Sized + 'static {
    /// Deserialize or otherwise convert an instance of `Self` from `response`.
    fn from_response<A>(adpt: &A, response: Response) -> Result<Self>
        where A: AbsAdapter;
}

impl<T> FromResponse for T where T: Deserialize + Send + 'static {
    fn from_response<A>(adpt: &A, mut response: Response) -> Result<Self>
        where A: AbsAdapter {
        adpt.deserializer().deserialize(&mut response)
    }
}

impl FromResponse for RawResponse {
    /// Simple wrapping operation; infallible.
    fn from_response<A>(_adpt: &A, response: Response) -> Result<Self>
        where A: AbsAdapter {

        Ok(RawResponse(response))
    }
}

/// Wrapper for `hyper::client::Response`.
///
/// Use this as a service method return type when you want to just get the raw response body from
/// a REST call.
///
/// Implements `Read` and `Into<hyper::client::Response>`.
pub struct RawResponse(pub Response);

impl Into<Response> for RawResponse {
    fn into(self) -> Response {
        self.0
    }
}

impl Read for RawResponse {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}