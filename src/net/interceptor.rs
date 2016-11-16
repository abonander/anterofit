//! Interceptors for common use-cases

pub use hyper::header;

use hyper::header::{Header, HeaderFormat, Headers};
use hyper::Url;

use std::borrow::Cow;
use std::collections::HashMap;

/// Easier, quicker to type since it's used a lot in these APIs.
pub type StaticCowStr = Cow<'static, str>;

use net::intercept::Interceptor;
use net::request::RequestHead;

/// An interceptor which adds the wrapped header to every request.
pub struct AddHeader<H: Header + HeaderFormat>(pub H);

impl<H: Header + HeaderFormat> Interceptor for AddHeader<H> {
    fn intercept(&self, req: &mut RequestHead) {
        req.header(self.0.clone());
    }
}

/// An interceptor which adds the contained headers to every request.
pub struct AddHeaders(pub Headers);

impl AddHeaders {
    pub fn new() -> Self {
        Headers::new().into()
    }

    pub fn header<H>(mut self, header: H) -> Self where H: Header + HeaderFormat {
        self.0.add(header);
        self
    }
}

impl From<Headers> for AddHeaders {
    fn from(headers: Headers) -> Self {
        AddHeaders(headers)
    }
}

impl Interceptor for AddHeaders {
    fn intercept(&self, req: &mut RequestHead) {
        req.headers(&self.0);
    }
}

pub struct PrependUrl(StaticCowStr);

impl PrependUrl {
    pub fn new<U: Into<Cow<'static, str>>>(url: U) {
        PrependUrl(url.into())
    }
}

impl Interceptor for PrependUrl {
    fn intercept(&self, req: &mut RequestHead) {
        req.prepend_url(&self.0);
    }
}

pub struct AppendUrl(StaticCowStr);

impl AppendUrl {
    pub fn new<U: Into<Cow<'static, str>>>(url: U) {
        AppendUrl(url.into())
    }
}

pub struct AppendQuery(Vec<(StaticCowStr, StaticCowStr)>);

impl AppendQuery {
    pub fn new() -> Self {
        AppendQuery(Vec::new())
    }

    pub fn pair<K, V>(mut self, key: K, val: V) -> Self
    where K: Into<StaticCowStr>, V: Into<StaticCowStr> {
        self.0.push((key.into(), val.into()));
        self
    }
}

impl Interceptor for AppendQuery {
    fn intercept(&self, req: &mut RequestHead) {
        req.query(&self.0);
    }
}