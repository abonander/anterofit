//! Types for modifying outgoing requests on-the-fly, e.g. to add headers or query parameters.

use hyper::header::{Header, HeaderFormat};

use super::RequestHead;

use std::borrow::Cow;

use std::fmt;

use std::sync::Arc;

impl fmt::Debug for Interceptor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug(f)
    }
}

impl<I: Interceptor + ?Sized> Interceptor for Arc<I> {
    fn intercept(&self, req: &mut RequestHead) {
        (**self).intercept(req)
    }
}

/// A trait describing a type which may intercept and modify outgoing request from an adapter
/// instance.
///
/// Implemented for `Fn(&mut RequestHead) + Send + Sync + 'static`.
pub trait Interceptor: Send + Sync + 'static {
    /// Modify the request head in any way desired.
    ///
    /// Great care must be taken to not introduce logic errors in service methods
    /// (i.e. by changing their endpoints such that they receive unexpected responses).
    fn intercept(&self, req: &mut RequestHead);

    /// Chain `self` with `then`, invoking `self` then `then` for each request.
    fn chain<I>(self, then: I) -> Chain<Self, I> where Self: Sized, I: Interceptor {
        Chain(self, then)
    }

    /// Chain `self` with two more interceptors.
    ///
    /// Saves a level in debug printing, mainly.
    fn chain2<I1, I2>(self, then: I1, after: I2) -> Chain2<Self, I1, I2> where Self: Sized,
                                                       I1: Interceptor, I2: Interceptor {

        Chain2(self, then, after)
    }

    /// Write debug output equivalent to `std::fmt::Debug`.
    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_debug(f)
    }

    /// Overridden by `NoIntercept`
    #[doc(hidden)]
    fn into_opt_obj(self) -> Option<Arc<Interceptor>> where Self: Sized {
        Some(Arc::new(self))
    }
}

impl<F> Interceptor for F where F: Fn(&mut RequestHead) + Send + Sync + 'static {
    fn intercept(&self, req: &mut RequestHead) {
        (*self)(req)
    }

    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("<(closure) as Interceptor>")
    }
}

/// Chains one interceptor with another, invoking them in declaration order.
#[derive(Debug)]
pub struct Chain<I1, I2>(I1, I2);

impl<I1: Interceptor, I2: Interceptor> Interceptor for Chain<I1, I2> {
    fn intercept(&self, req: &mut RequestHead) {
        self.0.intercept(req);
        self.1.intercept(req);
    }

    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Chain")
            .field(&(&self.0 as &Interceptor))
            .field(&(&self.1 as &Interceptor))
            .finish()
    }
}

/// Chains one interceptor with two more, invoking them in declaration order.
#[derive(Debug)]
pub struct Chain2<I1, I2, I3>(I1, I2, I3);

impl<I1: Interceptor, I2: Interceptor, I3: Interceptor> Interceptor for Chain2<I1, I2, I3> {
    fn intercept(&self, req: &mut RequestHead) {
        self.0.intercept(req);
        self.1.intercept(req);
        self.2.intercept(req);
    }

    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Chain2")
            .field(&(&self.0 as &Interceptor))
            .field(&(&self.1 as &Interceptor))
            .field(&(&self.2 as &Interceptor))
            .finish()
    }
}

/// A no-op interceptor which does nothing when invoked.
#[derive(Debug)]
pub struct NoIntercept;

impl Interceptor for NoIntercept {
    fn intercept(&self, _req: &mut RequestHead) {}

    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }

    fn into_opt_obj(self) -> Option<Arc<Interceptor>> {
        None
    }
}

/// Adds the wrapped header to every request.
///
/// To add multiple headers to one request, chain this interceptor with another.
#[derive(Debug)]
pub struct AddHeader<H: Header + HeaderFormat>(pub H);

impl<H: Header + HeaderFormat> Interceptor for AddHeader<H> {
    fn intercept(&self, req: &mut RequestHead) {
        req.header(self.0.clone());
    }

    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

/// Prepends the given string to every request's URL.
///
/// This is done *before* the adapter prepends the base URL. To override the base URL,
/// use a different adapter.
pub struct PrependUrl<S>(pub S);

impl<S: AsRef<str> + Send + Sync + 'static> Interceptor for PrependUrl<S> {
    fn intercept(&self, req: &mut RequestHead) {
        req.prepend_url(&self.0);
    }

    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("PrependUrl")
            .field(&self.0.as_ref())
            .finish()
    }
}

/// Appends the given string to every request's URL.
///
/// This is done *before* the adapter prepends the base URL. To override the base URL,
/// use a different adapter.
#[derive(Debug)]
pub struct AppendUrl<S>(pub S);

impl<S: AsRef<str> + Send + Sync + 'static> Interceptor for AppendUrl<S> {
    fn intercept(&self, req: &mut RequestHead) {
        req.append_url(self.0.as_ref());
    }

    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("AppendUrl")
            .field(&self.0.as_ref())
            .finish()
    }
}

/// Appends the given query pairs to every request.
///
/// Meant to be used in a builder style by calling `pair()` repeatedly.
///
/// This will not overwrite previous query pairs with the same key; it is left
/// to the server to decide which duplicate keys to keep.
pub struct AppendQuery(Vec<(Cow<'static, str>, Cow<'static, str>)>);

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
        where K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>> {
        self.pair_mut(key, val);
        self
    }

    /// Add a query key-value pair to this interceptor. Returns `&mut self` for builder-style usage.
    ///
    /// `key` and `val` can be any of: `String`, `&'static str` or `Cow<'static, str>`.
    pub fn pair_mut<K, V>(&mut self, key: K, val: V) -> &mut Self
        where K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>> {
        self.0.push((key.into(), val.into()));
        self
    }
}

impl Interceptor for AppendQuery {
    fn intercept(&self, req: &mut RequestHead) {
        req.query(self.0.iter());
    }

    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
         f.debug_map().entries(self.0.iter().map(|&(ref k, ref v)| (&**k, &**v))).finish()
    }
}

/// Specialized version of `fmt::Debug`
trait InterceptDebug {
    fn fmt_debug(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

#[cfg(not(feature = "nightly"))]
impl<T> InterceptDebug for T {
    fn fmt_debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Interceptor")
    }
}

#[cfg(feature = "nightly")]
mod nightly {
    use std::fmt;

    use super::InterceptDebug;

    impl<T> InterceptDebug for T {
        default fn fmt_debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("Interceptor")
        }
    }

    impl<T: fmt::Debug> InterceptDebug for T {
        fn fmt_debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
            <Self as fmt::Debug>::fmt(self, f)
        }
    }
}