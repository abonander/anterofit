//! Types for constructing and issuing HTTP requests.

use hyper::client::{Client, Response, RequestBuilder as NetRequestBuilder};
use hyper::header::{Headers, Header, HeaderFormat, ContentType};
use hyper::method::Method as HyperMethod;

use url::Url;
use url::form_urlencoded::Serializer as FormUrlEncoded;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

use std::borrow::{Borrow, Cow};
use std::fmt::{self, Write};
use std::mem;

use adapter::{AbsAdapter, AdapterConsts};

use mpmc::Sender;

use net::body::{Body, EmptyFields, EagerBody, RawBody};

use net::call::Call;

use net::intercept::Interceptor;

use net::method::{Method, TakesBody};

use net::response::FromResponse;

use executor::ExecBox;

use serialize::{Serializer, Deserializer};

use ::Result;

/// The request header, containing all the information needed to initialize a request.
#[derive(Debug)]
pub struct RequestHead {
    url: Cow<'static, str>,
    query: String,
    method: HyperMethod,
    headers: Headers
}

impl RequestHead {
    fn new(method: HyperMethod, url: Cow<'static, str>) -> Self {
        RequestHead {
            url: url.into(),
            query: String::new(),
            method: method,
            headers: Headers::new(),
        }
    }

    /// Set an HTTP header for this request, overwriting any previous value.
    ///
    /// ##Note
    /// Some headers, such as `Content-Type`, may be overwritten by Anterofit.
    pub fn header<H: Header + HeaderFormat>(&mut self, header: H) -> &mut Self {
        self.headers.set(header);
        self
    }

    /// Copy all the HTTP headers from `headers` into this request.
    ///
    /// Duplicate headers will be overwritten.
    ///
    /// ##Note
    /// Some headers, such as `Content-Type`, may be overwritten by Anterofit.
    pub fn headers(&mut self, headers: &Headers) -> &mut Self {
        self.headers.extend(headers.iter());
        self
    }

    /// Append `append` to the URL of this request.
    ///
    /// If this causes the request's URL to be malformed, an error will immediately
    /// be returned by `init_request()`.
    ///
    /// Characters that are not allowed to appear in a URL will be percent-encoded as appropriate
    /// for the path section of a URL.
    ///
    /// ## Note
    /// Adding a query segment via this method will not work as `?` and `=` will be encoded. Use
    /// `query()` instead to add query pairs.
    pub fn append_url<A: AsRef<str>>(&mut self, append: A) -> &mut Self {
        self.url.to_mut().extend(utf8_percent_encode(append.as_ref(), DEFAULT_ENCODE_SET));
        self
    }

    /// Prepend `prepend` to the URL of this request.
    ///
    /// This will appear between the current request URL and the base URL supplied by the adapter,
    /// if present, as the base URL is not appended until `init_request()`.
    ///
    /// If this causes the request's URL to be malformed, an error will immediately
    /// be returned by `init_request()`.
    ///
    /// Characters that are not allowed to appear in a URL will not be automatically percent-encoded.
    pub fn prepend_url<P: AsRef<str>>(&mut self, prepend: P) -> &mut Self {
        prepend_str(prepend.as_ref(), self.url.to_mut());
        self
    }

    /// Add a series of key-value pairs to this request's query. These will appear in the request
    /// URL.
    ///
    /// Characters that are not allowed to appear in a URL will automatically be percent-encoded.
    ///
    /// It is left up to the server how to resolve duplicate keys.
    ///
    /// Thanks to the mess of generics, this method is incredibly flexible: you can pass a reference
    /// to an array of pairs (2-tuples), a vector of pairs, a `HashMap` or `BTreeMap`, or any other
    /// iterator that yields pairs or references to pairs.
    ///
    /// ```rust,no_run
    /// # extern crate anterofit;
    /// # use std::collections::HashMap;
    /// # let head: &mut anterofit::net::RequestHead = unimplemented!();
    /// // `head` is `&mut RequestHead`
    /// head.query(&[("hello", "world"), ("id", "3")]);
    ///
    /// let query_pairs: HashMap<String, String> = HashMap::new();
    ///
    /// // Add some items to the map (...)
    /// head.query(query_pairs);
    /// ```
    ///
    /// ##Panics
    /// If an error is returned from `<K as Display>::fmt()` or `<V as Display>::fmt()`.
    pub fn query<Q, P, K, V>(&mut self, query: Q) -> &mut Self
    where Q: IntoIterator<Item=P>, P: Borrow<(K, V)>, K: fmt::Display, V: fmt::Display {
        let mut query_out = FormUrlEncoded::new(mem::replace(&mut self.query, String::new()));

        let mut kbuf = String::new();
        let mut vbuf = String::new();

        for pair in query {
            let &(ref key, ref val) = pair.borrow();

            kbuf.clear();
            vbuf.clear();

            // Errors here should be rare and usually indicate more serious problems.
            let _ = write!(kbuf, "{}", key).expect("Error returned from Display::fmt()");
            let _ = write!(vbuf, "{}", val).expect("Error returned from Display::fmt()");

            query_out.append_pair(&kbuf, &vbuf);
        }

        self.query = query_out.finish();

        self
    }

    /// Initialize a `hyper::client::RequestBuilder` with the parameters in this header.
    ///
    /// If provided, `base_url` will be prepended to the URL associated with this request,
    /// *then* the constructed query will be set to the URL.
    ///
    /// Finally, `client` will be used to create the `RequestBuilder` and the contained headers
    /// will be added.
    pub fn init_request<'c>(&self, base_url: Option<&Url>, client: &'c Client) -> Result<NetRequestBuilder<'c>> {
        let mut url = if let Some(base_url) = base_url {
            try!(base_url.join(&self.url))
        } else {
            try!(Url::parse(&self.url))
        };

        url.set_query(Some(&self.query));

        // This `.clone()` should be zero-cost, we don't expose Method::Extension at all.
        Ok(client.request(self.method.clone(), url).headers(self.headers.clone()))
    }

    /// Get the current URL of this request.
    pub fn get_url(&self) -> &str {
        &self.url
    }

    /// Get the current query string of this request.
    pub fn get_query(&self) -> &str {
        &self.query
    }

    /// Get the HTTP method of this request.
    pub fn get_method(&self) -> &HyperMethod {
        &self.method
    }

    /// Get the headers of this request (may be modified later).
    pub fn get_headers(&self) -> &Headers {
        &self.headers
    }
}

impl fmt::Display for RequestHead {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}{}", self.method, self.url, self.query)
    }
}

/// A container for a request header and body.
///
/// Used in the body of service methods to construct a request.
#[derive(Debug)]
pub struct RequestBuilder<'a, A: 'a + ?Sized, M, B> {
    head: RequestHead,
    method: M,
    body: B,
    adapter: &'a A,
}

impl<'a, A: 'a + ?Sized, M> RequestBuilder<'a, A, M, EmptyFields> where M: Method {
    /// Create a new request builder with the given method and URL.
    ///
    /// `url` can be `String` or `&'static str`.
    pub fn new(adapter: &'a A, method: M, url: Cow<'static, str>) -> Self {
        RequestBuilder {
            adapter: adapter,
            head: RequestHead::new(method.to_hyper(), url),
            method: method,
            body: EmptyFields,
        }
    }
}

impl<'a, A: 'a + ?Sized, M, B> RequestBuilder<'a, A, M, B> {
    /// Get a reference to the header of the request to inspect it.
    pub fn head(&self) -> &RequestHead {
        &self.head
    }

    /// Get a mutable reference to the header of the request.
    ///
    /// Can be used to change the request URL, add GET query pairs or HTTP headers to be
    /// sent with the request.
    pub fn head_mut(&mut self) -> &mut RequestHead {
        &mut self.head
    }

    /// Pass `self` to the closure, allowing it to mutate and transform the builder
    /// arbitrarily.
    ///
    /// `try!()` will work in this closure.
    pub fn apply<F, B_>(self, functor: F) -> Result<RequestBuilder<'a, A, M, B_>>
    where F: FnOnce(Self) -> Result<RequestBuilder<'a, A, M, B_>> {
        functor(self)
    }
}

impl<'a, A: 'a + ?Sized, M, B> RequestBuilder<'a, A, M, B> where A: AbsAdapter, M: TakesBody {
    /// Set a body to be sent with the request.
    ///
    /// Generally, `GET` and `DELETE` are not to have bodies
    // If you need to have a body on a GET or DELETE request
    pub fn body<B_>(self, body: B_) -> RequestBuilder<'a, A, M, B_> {
        RequestBuilder {
            adapter: self.adapter,
            head: self.head,
            method: self.method,
            body: body,
        }
    }

    /// Immediately serialize `body` on the current thread and set the result as the body
    /// of this request.
    ///
    /// This is useful if you want to use a body type that is not `Send` or `'static`.
    ///
    /// ##Panics
    /// If this is a GET request (cannot have a body).
    pub fn body_eager<B_>(self, body: B_)
        -> Result<RequestBuilder<'a, A, M, RawBody<<B_ as EagerBody>::Readable>>>
        where B_: EagerBody {

        let body = try!(body.into_readable(&self.adapter.ref_consts().serializer)).into();
        Ok(self.body(body))
    }
}

impl<'a, A: 'a + ?Sized, M, B> RequestBuilder<'a, A, M, B> where A: AbsAdapter {
    /// Prepare a `Request` to be executed with the parameters supplied in this builder.
    ///
    /// This request will need to be executed (using `exec()` or `exec_here()`) before anything
    /// else is done. As much work as possible will be relegated to the adapter's executor.
    pub fn build<T>(self) -> Request<'a, T> where B: Body, T: FromResponse {
        let RequestBuilder {
            adapter, head, method: _method, body
        } = self;

        let consts = adapter.consts();
        let interceptor = adapter.interceptor();

        let (mut guard, call) = super::call::oneshot(Some(head));

        let exec = ExecRequest {
            sender: &adapter.ref_consts().sender,
            exec: Box::new(move || {
                let interceptor = interceptor.as_ref().map(|i| &**i);

                let res = exec_request(&consts, interceptor, guard.head_mut(), body)
                    .and_then(|response| T::from_response(&consts.deserializer, response));

                guard.complete(res);
            }),
        };

        Request {
            exec: Some(exec),
            call: call,
        }
    }

    /// Equivalent to `body()` but is not restricted from `GET` or `DELETE` requests.
    pub fn force_body<B_>(self, body: B_) -> RequestBuilder<'a, A, M, B_> {
        RequestBuilder {
            adapter: self.adapter,
            head: self.head,
            method: self.method,
            body: body,
        }
    }

    /// Equivalent to `body_eager()` but is not restricted from `GET` or `DELETE` requests.
    pub fn force_body_eager<B_>(self, body: B_)
        -> Result<RequestBuilder<'a, A, M, RawBody<<B_ as EagerBody>::Readable>>>
        where B_: EagerBody {

        let body = try!(body.into_readable(&self.adapter.ref_consts().serializer)).into();
        Ok(self.force_body(body))
    }
}

struct ExecRequest<'a> {
    sender: &'a Sender,
    exec: Box<ExecBox>,
}

impl<'a> ExecRequest<'a> {
    fn exec(self) {
        self.sender.send(self.exec);
    }

    fn exec_here(self) {
        self.exec.exec();
    }
}

/// A request which is ready to be sent to the server.
///
/// Use `exec()` or `exec_here()` to send the request and receive the response.
///
/// ##Note
/// If an error occurred during initialization of the request, it will be immediately
/// returned when the request is executed; no network or disk activity will occur.
#[must_use = "Request has not been sent yet"]
pub struct Request<'a, T = ()> {
    exec: Option<ExecRequest<'a>>,
    call: Call<T>,
}

impl<'a, T> Request<'a, T> {
    /// Construct a `Result` wrapping an immediate return of `res`.
    ///
    /// No network or disk activity will occur when this request is executed.
    pub fn immediate(res: Result<T>) -> Request<'static, T> {
        Request {
            exec: None,
            call: super::call::immediate(res),
        }
    }

    /// Execute this request on the current thread, **blocking** until the result is available.
    pub fn exec_here(self) -> Result<T> {
        self.exec.map(ExecRequest::exec_here);
        self.call.block()
    }

    /// Returns `true` if a result is immediately available (`exec_here()` will not block).
    pub fn is_immediate(&self) -> bool {
        self.call.is_immediate()
    }
}

impl<'a, T> Request<'a, T> where T: Send + 'static {
    /// Execute this request on the adapter's executor, returning a type which can
    /// be polled for the result.
    pub fn exec(self) -> Call<T> {
        self.exec.map(ExecRequest::exec);
        self.call
    }

    /// Add a callback to be executed with the request's return value when available, mapping
    /// it to another value (or `()` if no return value).
    ///
    /// `on_complete` will always be executed on the adapter's executor because the return
    /// value will not be available until the request is executed, whereas `on_result()`'s closure
    /// may be executed immediately if an immediate result is available.
    ///
    /// If a result is immediately available, `on_complete` will be discarded.
    ///
    /// ## Note
    /// `on_complete` should not be long-running in order to not block other requests waiting
    /// on the executor.
    ///
    /// ## Warning about Panics
    /// Panics in `on_complete` will cause the return value to be lost. There is no safety
    /// issue and subsequent requests shouldn't be affected, but it may be harder to debug
    /// without knowing which request caused the panic.
    #[cfg(any())]
    pub fn on_complete<F, R>(self, on_complete: F) -> Request<'a, R>
    where F: FnOnce(T) -> R + Send + 'static, R: Send + 'static {
        self.on_result(|res| res.map(on_complete))
    }

    // RFC: add `on_error()`?

    /// Add a callback to be executed with the request's result when available, mapping it to
    /// another result (which can be `::Result<()>`).
    ///
    /// If a result is immediately available, `on_result` will be executed on the current thread
    /// with the result, and the return value will be immediately available as well.
    ///
    /// ## Note
    /// `on_result` should not be long-running in order to not block other requests waiting
    /// on the executor, or block the current thread if the result is immediate.
    ///
    /// ## Warning about Panics
    /// Panics in `on_result` will cause the return value to be lost. There is no safety
    /// issue and subsequent requests shouldn't be affected, but it may be harder to debug
    /// without knowing which request caused the panic.
    ///
    /// If the result is immediately available, panics in `on_result` will occur on the
    /// current thread.
    #[cfg(any())]
    pub fn on_result<F, R>(self, on_result: F) -> Request<'a, R>
    where F: FnOnce(Result<T>) -> Result<R> + Send + 'static, R: Send + 'static {
        let Request { exec, call } = self;

        if call.is_immediate() {
            let res = on_result(call.block());
            return Request::immediate(res);
        }

        let exec = exec.expect("`self.exec` was `None` when it shouldn't be");

        let (mut guard, new_call) = super::call::oneshot(None);

        let new_exec = ExecRequest {
            sender: exec.sender,
            exec: Box::new(move || {
                exec.exec();

                guard.complete(
                    on_result(call.block())
                );
            })
        };

        Request {
            exec: Some(new_exec),
            call: new_call,
        }
    }
}

fn exec_request<S, D, B>(consts: &AdapterConsts<S, D>, interceptor: Option<&Interceptor>, head: &mut RequestHead, body: B) -> Result<Response>
where S: Serializer, D: Deserializer, B: Body {
    if let Some(interceptor) = interceptor {
        interceptor.intercept(head);
    }

    let mut readable = try!(body.into_readable(&consts.serializer));

    if let Some(content_type) = readable.content_type {
        head.header(ContentType(content_type));
    }

    head.init_request(consts.base_url.as_ref(), &consts.client)?
        .body(&mut readable.readable).send().map_err(Into::into)
}

fn prepend_str(prepend: &str, to: &mut String) {
    to.insert_str(0, prepend);
}
