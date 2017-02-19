//! Strongly typed HTTP methods and their traits

macro_rules! method (
    ($(#[$meta:meta])* pub struct $method:ident) => (
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $method;

        impl Method for $method {
            fn to_hyper(&self) -> ::hyper::method::Method {
                ::hyper::method::Method::$method
            }
        }
    );
    ($($(#[$meta:meta])* pub struct $method:ident);+;) =>(
        $(method!($(#[$meta])* pub struct $method);)+
    )
);

macro_rules! takes_body (
    ($method:ident) => (
        impl TakesBody for $method {}
    );
    ($($method:ident),+) => (
        $(takes_body!($method);)+
    );
);

method! {
    /// Method for `GET` requests.
    ///
    /// ### Note: Body
    /// While `GET` requests are not forbidden to have a body by the HTTP spec,
    /// it is not meaningful to provide a body with a `GET` request and any endpoint
    /// that expects a body with such a request is considered non-conformant.
    pub struct Get;
    /// Method for `POST` requests, can take a body.
    pub struct Post;
    /// Method for `PUT` requests, can take a body.
    pub struct Put;
    /// Method for `PATCH` requests, can take a body.
    pub struct Patch;
    /// Method for `DELETE` requests.
    ///
    /// ### Note: Body
    /// While `DELETE` requests are not forbidden to have a body by the HTTP spec,
    /// it is not meaningful to provide a body with a `DELETE` request and any endpoint
    /// that expects a body with such a request is considered non-conformant.
    pub struct Delete;
}

takes_body! { Post, Put, Patch }

/// The HTTP method of a request in Anterofit.
pub trait Method {
    /// Convert to Hyper's `Method` enum.
    fn to_hyper(&self) -> ::hyper::method::Method;
}

/// Trait describing an HTTP method which is allowed to have a body.
///
/// ### Motivation
/// Though not forbidden in the HTTP spec, `GET` and `DELETE` requests are generally not
/// meant to have bodies, and it is recommended for servers to ignore bodies on such
/// requests ([IETF RFC 2616 (HTTP 1.1 Spec), Section 4.3][rfc2616-4.3]).
///
/// Thus, this trait acts as a strongly typed anti-footgun for when you specified a
/// `GET` or `DELETE` request when you actually meant `POST` or `PUT` or `PATCH`.
///
/// If you must have a body on a `GET` or `DELETE` request, you can use the
/// `force_body()` or `force_body_eager()` methods on `RequestBuilder` in conjunction
/// with `map_builder!()` in a service method definition.
///
/// [rfc2616-4.3]: https://tools.ietf.org/html/rfc2616#section-4.3
pub trait TakesBody {}