//! Types concerning the responses from REST calls.

pub use hyper::client::Response;

use std::io::{self, Read};

use serialize::{Deserialize, Deserializer};

use Result;

/// A trait describing types which can be converted from raw response bodies.
///
/// Implemented for `T: Deserialize + Send + 'static`.
///
/// Use `response::Raw` if you just want the response body, or `WithRaw` or `TryWithRaw`
/// if you want the response body and the deserialized value.
pub trait FromResponse: Send + Sized + 'static {
    /// Deserialize or otherwise convert an instance of `Self` from `response`.
    fn from_response<D>(des: &D, response: Response) -> Result<Self>
    where
        D: Deserializer;
}

impl<T> FromResponse for T
where
    T: Deserialize + Send + 'static,
{
    fn from_response<D>(des: &D, mut response: Response) -> Result<Self>
    where
        D: Deserializer,
    {
        des.deserialize(&mut response)
    }
}

/// Wrapper for `hyper::client::Response`.
///
/// Use this as a service method return type when you want to just get the raw response body from
/// a REST call.
///
/// Implements `Read` and `Into<hyper::client::Response>`.
pub struct Raw(pub Response);

impl Into<Response> for Raw {
    fn into(self) -> Response {
        self.0
    }
}

impl Read for Raw {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl FromResponse for Raw {
    /// Simple wrapping operation; infallible.
    fn from_response<D>(_des: &D, response: Response) -> Result<Self>
    where
        D: Deserializer,
    {
        Ok(Raw(response))
    }
}

/// Wrapper for the parsed response value along with the raw response.
///
/// Use this as a service method return type when you want to inspect the response
/// after the true return value has been deserialized.
pub struct WithRaw<T> {
    /// The raw `hyper::client::Response` instance.
    ///
    /// ### Note
    /// The deserializer will likely have already read to the end of the HTTP stream. Use `Raw`
    /// if you want to read the response yourself.
    pub raw: Response,
    /// The deserialized value.
    pub value: T,
}

impl<T> FromResponse for WithRaw<T>
where
    T: Deserialize + Send + 'static,
{
    fn from_response<D>(des: &D, mut response: Response) -> Result<Self>
    where
        D: Deserializer,
    {
        let val = try!(des.deserialize(&mut response));
        Ok(WithRaw {
            raw: response,
            value: val,
        })
    }
}

/// Wrapper for the deserialization result along with the raw response.
///
/// Use this as a service method return type if you want the raw response whether
/// or not deserialization of the true return type succeeded.
pub struct TryWithRaw<T> {
    /// The raw `hyper::client::Response` instance.
    ///
    /// ### Note
    /// The deserializer will likely have already read to the end of the HTTP stream. Use `Raw`
    /// if you want to read the response yourself.
    pub raw: Response,
    /// The result of attempting to deserialize the value.
    pub result: Result<T>,
}

impl<T> FromResponse for TryWithRaw<T>
where
    T: Deserialize + Send + 'static,
{
    fn from_response<D>(des: &D, mut response: Response) -> Result<Self>
    where
        D: Deserializer,
    {
        let res = des.deserialize(&mut response);
        Ok(TryWithRaw {
            raw: response,
            result: res,
        })
    }
}

impl<T> Into<Result<T>> for TryWithRaw<T> {
    fn into(self) -> Result<T> {
        self.result
    }
}
