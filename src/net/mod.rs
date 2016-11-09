

pub use hyper::method::Method;

pub use hyper::header::Headers;

pub use self::adapter::{Adapter, RequestAdapter};

pub use self::intercept::{Interceptor, Chain};

pub use self::body::*;

pub use self::call::Call;

pub use self::request::{RequestHead, RequestBuilder, Request};

mod adapter;

mod body;

mod call;

mod intercept;

mod request;

use ::Result;


