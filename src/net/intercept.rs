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

/// Chains two interceptors together, invoking the first, then the second.
pub struct Chain<I1, I2>(I1, I2);

impl<I1: Interceptor, I2: Interceptor> Interceptor for Chain<I1, I2> {
    fn intercept(&self, req: &mut RequestHead) {
        self.0.intercept(req);
        self.1.intercept(req);
    }
}