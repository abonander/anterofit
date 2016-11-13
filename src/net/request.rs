use hyper::client::{Client, Response, RequestBuilder as NetRequestBuilder};
use hyper::header::{Headers, Header, HeaderFormat, ContentType};
use hyper::method::Method;

use url::Url;
use url::form_urlencoded::Serializer as FormUrlEncoded;

use std::borrow::Cow;
use std::fmt::{self, Write};
use std::mem;

use net::adapter::RequestAdapter;

use net::body::{Body, EmptyFields};

use net::call::Call;

use net::response::FromResponse;

use executor::ExecBox;

use ::Result;

pub struct RequestHead {
    url: Cow<'static, str>,
    query: String,
    method: Method,
    headers: Headers
}

impl RequestHead {
    fn new<U: Into<Cow<'static, str>>>(method: Method, url: U) -> Self {
        RequestHead {
            url: url.into(),
            query: String::new(),
            method: method,
            headers: Headers::new(),
        }
    }

    pub fn header<H: Header + HeaderFormat>(&mut self, header: H) -> &mut Self {
        self.headers.set(header);
        self
    }

    pub fn append_url<A: AsRef<str>>(&mut self, append: A) -> &mut Self {
        *self.url.to_mut() += append.as_ref();
        self
    }

    pub fn prepend_url<P: AsRef<str>>(&mut self, prepend: P) -> &mut Self {
        prepend_str(prepend.as_ref(), self.url.to_mut());
        self
    }

    pub fn append_query(&mut self, query: &[(&fmt::Display, &fmt::Display)]) -> &mut Self {
        let mut query_out = FormUrlEncoded::new(mem::replace(&mut self.query, String::new()));

        let mut kbuf = String::new();
        let mut vbuf = String::new();

        for &(key, val) in query {
            kbuf.clear();
            vbuf.clear();

            // This will never error
            let _ = write!(kbuf, "{}", key);
            let _ = write!(vbuf, "{}", val);

            query_out.append_pair(&kbuf, &vbuf);
        }

        self.query = query_out.finish();

        self
    }

    pub fn init_request<'c>(self, base_url: Option<&Url>, client: &'c Client) -> Result<NetRequestBuilder<'c>> {
        let mut url = if let Some(base_url) = base_url {
            try!(base_url.join(&self.url))
        } else {
            try!(Url::parse(&self.url))
        };

        url.set_query(Some(&self.query));

        Ok(client.request(self.method, url).headers(self.headers))
    }
}

pub struct RequestBuilder<B> {
    head: RequestHead,
    body: B,
}

impl RequestBuilder<EmptyFields> {
    pub fn new<U: Into<Cow<'static, str>>>(method: Method, url: U) -> Self {
        RequestBuilder {
            head: RequestHead::new(method, url),
            body: EmptyFields,
        }
    }

    pub fn body<B>(self, body: B) -> RequestBuilder<B> {
        if let Method::Get = self.head.method {
            panic!("Cannot supply a body with GET requests!");
        }

        RequestBuilder {
            head: self.head,
            body: body,
        }
    }
}

impl<B> RequestBuilder<B> {
    pub fn head_mut(&mut self) -> &mut RequestHead {
        &mut self.head
    }
}

#[must_use = "Request has not been sent yet"]
pub struct Request<'a, A: 'a, T> {
    adapter: &'a A,
    exec: Box<ExecBox>,
    call: Call<T>,
}

impl<'a, A: 'a, T> Request<'a, A, T> {
    /// Construct a `Result` wrapping an immediate return of `res`.
    ///
    /// No network or activity will occur when this request is executed.
    pub fn immediate(adapter: &'a A, res: Result<T>) -> Self {
        Request {
            adapter: adapter,
            exec: ExecBox::noop(),
            call: super::call::immediate(res),
        }
    }

    /// Execute this request on the current thread, **blocking** until the result is available.
    ///
    /// Does not require a valid adapter type.
    pub fn exec_here(self) -> Result<T> {
        self.exec.exec();
        self.call.block()
    }

    /// Returns `true` if a result is immediately available (`exec_here()` will not block).
    pub fn is_immediate(&self) -> bool {
        self.call.is_immediate()
    }
}

impl<'a, A: 'a, T> Request<'a, A, T> where A: RequestAdapter, T: FromResponse {
    /// Create a `Request` which is ready to be executed based on the parameters in `builder`
    /// and using the given adapter.
    ///
    /// This request will need to be executed (using `exec()` or `exec_here()`) before anything
    /// else is done. As much work as possible will be relegated to the adapter's executor.
    pub fn ready<B>(adpt: &A, builder: RequestBuilder<B>) -> Request<A, T>
        where B: Body {

        let adpt_ = adpt.clone();

        let (tx, rx) = ::futures::oneshot();

        let exec = Box::new(move ||
            tx.complete(
                exec_request(&adpt_, builder)
                    .and_then(|response| T::from_response(&adpt_, response))
            )
        );

        Request {
            adapter: adpt,
            exec: exec,
            call: super::call::from_oneshot(rx),
        }
    }

    /// Execute this request on the adapter's executor, returning a type which can
    /// be polled for the result.
    pub fn exec(self) -> Call<T> {
        self.adapter.execute(self.exec);
        self.call
    }
}

fn exec_request<A, B>(adpt: &A, mut builder: RequestBuilder<B>) -> Result<Response>
where A: RequestAdapter, B: Body{

    adpt.intercept(&mut builder.head);

    let mut readable = try!(builder.body.into_readable(adpt));

    if let Some(content_type) = readable.content_type {
        builder.head.header(ContentType(content_type));
    }

    let request = try!(adpt.request_builder(builder.head));

    request.body(&mut readable.readable).send().map_err(Into::into)
}

// FIXME: remove the inferior version and inline this when this is stabilized.
#[cfg(feature = "nightly")]
fn prepend_str(prepend: &str, to: &mut String) {
    to.insert_str(0, prepend);
}

// Stable workaround that avoids unsafe code at the cost of an additional allocation.
#[cfg(not(feature = "nightly"))]
fn prepend_str(prepend: &str, to: &mut String) {
    let cap = prepend.len().checked_add(to.len())
        .expect("Overflow evaluating capacity");

    let append = mem::replace(to, String::with_capacity(cap));

    *to += prepend;
    *to += &*append;
}
