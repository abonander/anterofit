
use futures::Complete;

use hyper::client::{Client, Response, RequestBuilder as NetRequestBuilder};
use hyper::error::Result as HyperResult;
use hyper::header::{Headers, Header, HeaderFormat, ContentType};
use hyper::method::Method;

use multipart::client::lazy::Multipart;

use url::{self, Url};
use url::form_urlencoded::Serializer as FormUrlEncoded;

use std::borrow::Cow;
use std::fmt::{self, Write};
use std::io::{self, Empty, Read};
use std::mem;

use std::panic;

use net::adapter::{Adapter, RequestAdapter, RequestAdapter_};

use net::body::{Body, EmptyFields};

use net::call::Call;

use net::intercept::Interceptor;

use executor::ExecBox;

use serialize::{Serialize, Deserialize};

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

    pub fn init_request<'c>(self, base_url: &Url, client: &'c Client) -> Result<NetRequestBuilder<'c>> {
        let mut url = try!(base_url.join(&self.url));
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
    pub fn immediate(adapter: &'a A, res: Result<T>) -> Self {
        Request {
            adapter: adapter,
            exec: ExecBox::noop(),
            call: super::call::immediate(res),
        }
    }
}

impl<'a, A: 'a, T> Request<'a, A, T> where A: RequestAdapter {
    pub fn async(self) -> Call<T> {
        self.adapter.execute(self.exec);
        self.call
    }

    pub fn here(self) -> Result<T> {
        self.exec.exec();
        self.call.block()
    }
}

pub fn new<A, B, T>(adpt: &A, builder: RequestBuilder<B>) -> Request<A, T>
where A: RequestAdapter, B: Body, T: Deserialize + Send + 'static {
    let adpt_ = adpt.clone();

    let (tx, rx) = ::futures::oneshot();

    let exec = Box::new(move || {
        tx.complete(exec_request(&adpt_, builder))
    });

    Request {
        adapter: adpt,
        exec: exec,
        call: super::call::from_oneshot(rx),
    }
}

fn exec_request<A, B, T>(adpt: &A, mut builder: RequestBuilder<B>) -> Result<T>
where A: RequestAdapter, B: Body, T: Deserialize + 'static {

    adpt.intercept(&mut builder.head);

    let mut readable = try!(builder.body.into_readable(adpt));

    if let Some(content_type) = readable.content_type {
        builder.head.header(ContentType(content_type));
    }

    let request = try!(adpt.request_builder(builder.head));

    let mut response = try!(request.body(&mut readable.readable).send());

    adpt.deserialize(&mut response)
}

// FIXME: remove the inferior version and inline this when this stabilized.
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
