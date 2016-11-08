pub use hyper::Error as HyperError;
pub use hyper::error::ParseError as UrlError;

use std::any::Any;
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
        },
        Url(e: UrlError) {
            from()
            cause(e)
            description(e.description())
        },
        #[cfg(feature = "json")]
        JsonSerialize(e: ::serialize::json::Error) {
            from()
            cause(e)
            description(e.description())
        },
        Io(e: IoError){
            from()
            cause(e)
            description(e.description())
        },
        Other(e: Box<StdError + Send>){
            from()
            cause(&**e)
            description(e.description())
        },
        Panic(e: Box<Any + Send>) {
            from()
            description(panic_err_str(e))
        }
        ResultTaken() {
            from(::futures::Canceled)
            description("The result has already been taken from this Call.")
        }
        /// Here to satisfy the type-checker.
        __Never(_: Never) {
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

fn panic_err_str(any: &(Any + Send)) -> &str {
    if let Some(s) = any.downcast_ref::<&'static str>() {
        s
    } else if let Some(s) = any.downcast_ref::<String>() {
        s
    } else {
        "Unexpected value passed to `panic!()` from the worker thread. Check the stderr stream
        for the panic backtrace."
    }
}