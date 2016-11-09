pub use hyper::Error as HyperError;

pub use url::ParseError as UrlError;
//pub use hyper::error::ParseError as UrlError;

use serialize::{NoSerializerError, NoDeserializerError};

pub type MultipartError = ::multipart::client::lazy::LazyIoError<'static>;

use std::io::Error as IoError;
use std::error::Error as StdError;
use std::fmt;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Hyper(e: HyperError) {
            from()
            cause(e)
            description(e.description())
        }
        Url(e: UrlError) {
            from()
            cause(e)
            description(e.description())
        }
        JsonSerialize(e: ::serialize::json::Error) {
            from()
            cause(e)
            description(e.description())
        }
        Io(e: IoError){
            from()
            cause(e)
            description(e.description())
        }
        Multipart(e: MultipartError) {
            from()
            cause(e)
            description(e.description())
        }
        NoSerializer(e: NoSerializerError) {
            from()
            cause(e)
            description(e.description())
        }
        NoDeserializer(e: NoDeserializerError) {
            from()
            cause(e)
            description(e.description())
        }
        Other(e: Box<StdError + Send>){
            from()
            cause(&**e)
            description(e.description())
        }
        Panic {
            from(::futures::Canceled)
            description("The request could not be completed because a panic occurred on the worker thread.")
        }
        ResultTaken {
            description("The result has already been taken from this Call.")
        }
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

pub fn flatten_res<T, E>(res: Result<Result<T, Error>, E>) -> Result<T, Error> where Error: From<E> {
    try!(res)
}

pub fn map_res<T, E>(res: Result<T, E>) -> Result<T, Error> where Error: From<E> {
    res.map_err(|e| e.into())
}