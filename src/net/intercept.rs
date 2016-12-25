//! Types for modifying outgoing requests on-the-fly, e.g. to add headers or query parameters.

use hyper::header::{Header, HeaderFormat};

use super::RequestHead;

/// A trait describing a type which may intercept and modify outgoing request from an adapter
/// instance.
///
/// Implemented for `Fn(&mut RequestHead) + Send + Sync + 'static`.
pub trait Interceptor: Send + Sync + 'static {
    /// Modify the request headers in any way desired.
    ///
    /// Great care must be taken to not introduce logic errors in service methods
    /// (i.e. by changing their endpoints such that they receive unexpected responses).
    fn intercept(&self, req: &mut RequestHead);

    /// Chain `self` with `other`, invoking `self` then `other` for each request.
    fn chain<I>(self, other: I) -> Chain<Self, I> where Self: Sized, I: Interceptor {
        Chain(self, other)
    }
}

/// Chains two interceptors together, invoking the first, then the second.
pub struct Chain<I1, I2>(I1, I2);

impl<I1: Interceptor, I2: Interceptor> Interceptor for Chain<I1, I2> {
    fn intercept(&self, req: &mut RequestHead) {
        self.0.intercept(req);
        self.1.intercept(req);
    }
}

use std::borrow::Cow;

/// Easier, quicker to type since it's used a lot in these APIs.
pub type StaticCowStr = Cow<'static, str>;
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