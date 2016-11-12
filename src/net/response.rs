pub use hyper::client::Response;

use std::io::{self, Read};

use serialize::Deserialize;

use super::adapter::RequestAdapter;

use ::Result;

pub trait FromResponse: Send + Sized + 'static {
    fn from_response<A>(adpt: &A, response: Response) -> Result<Self>
        where A: RequestAdapter;
}

impl<T> FromResponse for T where T: Deserialize + Send + 'static {
    fn from_response<A>(adpt: &A, mut response: Response) -> Result<Self>
        where A: RequestAdapter {
        adpt.deserialize(&mut response)
    }
}

impl FromResponse for RawResponse {
    fn from_response<A>(_adpt: &A, response: Response) -> Result<Self>
        where A: RequestAdapter {

        Ok(RawResponse(response))
    }
}

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