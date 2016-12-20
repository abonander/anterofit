//! Interceptors for common use-cases

use hyper::header::{Header, HeaderFormat};

use std::borrow::Cow;

/// Easier, quicker to type since it's used a lot in these APIs.
pub type StaticCowStr = Cow<'static, str>;

use net::intercept::Interceptor;
use net::request::RequestHead;

/// A no-op interceptor which does nothing when invoked.
pub struct NoIntercept;

impl Interceptor for NoIntercept {
    fn intercept(&self, _req: &mut RequestHead) {}
}

impl<F> Interceptor for F where F: Fn(&mut RequestHead) + Send + Sync + 'static {
    fn intercept(&self, req: &mut RequestHead) {
        (*self)(req)
    }
}

/// Adds the wrapped header to every request.
///
/// To add multiple headers to one request, chain this interceptor with another.
pub struct AddHeader<H: Header + HeaderFormat>(pub H);

impl<H: Header + HeaderFormat> Interceptor for AddHeader<H> {
    fn intercept(&self, req: &mut RequestHead) {
        req.header(self.0.clone());
    }
}

/// Prepends the given string to every request's URL.
///
/// This is done *before* the adapter prepends the base URL. To override the base URL,
/// use a different adapter.
pub struct PrependUrl(StaticCowStr);

impl PrependUrl {
    /// Wrap a `String` or `&'static str` or `Cow<'static, str>`.
    pub fn new<U: Into<StaticCowStr>>(url: U) -> Self {
        PrependUrl(url.into())
    }
}

impl Interceptor for PrependUrl {
    fn intercept(&self, req: &mut RequestHead) {
        req.prepend_url(&self.0);
    }
}

/// Appends the given string to every request's URL.
///
/// This is done *before* the adapter prepends the base URL. To override the base URL,
/// use a different adapter.
pub struct AppendUrl(StaticCowStr);

impl AppendUrl {
    /// Wrap a `String` or `&'static str` or `Cow<'static, str>`.
    pub fn new<U: Into<StaticCowStr>>(url: U) -> Self {
        AppendUrl(url.into())
    }
}

/// Appends the given query pairs to every request.
///
/// Meant to be used in a builder style by calling `pair()` repeatedly.
///
/// This will not overwrite previous query pairs with the same key; it is left
/// to the server to decide which duplicate keys to keep.
pub struct AppendQuery(Vec<(StaticCowStr, StaticCowStr)>);

impl AppendQuery {
    /// Create an empty vector of pairs.
    ///
    /// Meant to be used in a builder style.
    pub fn new() -> Self {
        AppendQuery(Vec::new())
    }

    /// Add a query key-value pair to this interceptor. Returns `self` for builder-style usage.
    ///
    /// `key` and `val` can be any of: `String`, `&'static str` or `Cow<'static, str>`.
    pub fn pair<K, V>(mut self, key: K, val: V) -> Self
    where K: Into<StaticCowStr>, V: Into<StaticCowStr> {
        self.pair_mut(key, val);
        self
    }

    /// Add a query key-value pair to this interceptor. Returns `&mut self` for builder-style usage.
    ///
    /// `key` and `val` can be any of: `String`, `&'static str` or `Cow<'static, str>`.
    pub fn pair_mut<K, V>(&mut self, key: K, val: V) -> &mut Self
    where K: Into<StaticCowStr>, V: Into<StaticCowStr> {
        self.0.push((key.into(), val.into()));
        self
    }
}

impl Interceptor for AppendQuery {
    fn intercept(&self, req: &mut RequestHead) {
        req.query(self.0.iter());
    }
}