
use futures::Complete;

use hyper::client::{IntoUrl, Response, RequestBuilder as NetRequestBuilder};
use hyper::error::Result as HyperResult;
use hyper::header::{Headers, Header, HeaderFormat};
use hyper::method::Method;

use multipart::client::lazy::Multipart;

use url::{self, Url};

use std::fmt::{self, Write};
use std::io::{self, Empty, Read};
use std::mem;

use std::panic;

use net::{Adapter, RequestAdapter, RequestAdapter_};

use net::body::Body;

use net::intercept::Interceptor;

use ::{ExecBox, Result};

pub struct RequestHead {
    url: Url,
    query: Url,
    method: Method,
    headers: Headers
}

impl RequestHead {
    fn new(method: Method, url: Url) -> Self {
        RequestHead {
            url: url,
            query: Url::parse("").unwrap(),
            method: method,
            headers: Headers::new(),
        }
    }

    pub fn header<H: Header + HeaderFormat>(&mut self, header: H) -> &mut Self {
        self.headers.set(header);
        self
    }

    pub fn append_url(&mut self, append: &Url) -> &mut Self {

        self
    }

    pub fn prepend_url(&mut self, prepend: Url) -> &mut Self {
        let append = mem::replace(&mut self.url, prepend);
        self.append_url(&append);
        self
    }

    pub fn append_query(&mut self, query: &[(&fmt::Display, &fmt::Display)]) -> &mut Self {
        {
            let mut kbuf = String::new();
            let mut vbuf = String::new();

            let mut query_out = self.query.query_pairs_mut();

            for &(key, val) in query {
                kbuf.clear();
                vbuf.clear();

                // This will never error
                let _ = write!(kbuf, "{}", key);
                let _ = write!(vbuf, "{}", val);

                query_out.append_pair(&kbuf, &vbuf);
            }

            let _ = query_out.finish();
        }

        self
    }

    fn start()
}

pub struct RequestBuilder<B> {
    head: RequestHead,
    body: B,
}

impl RequestBuilder<()> {
    pub fn new(method: Method, url: Url) -> Self {
        RequestBuilder {
            head: RequestHead::new(method, url),
            body: (),
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
pub struct Request<'a, A, T> {
    adapter: &'a A,
    exec: Box<ExecBox>,
    call: Call<T>,
}

impl<'a, A, T> Request<'a, A, T> {
    pub fn here(self) -> Result<T> {

    }
}

pub fn new<A, B, T>(adpt: &A, builder: RequestBuilder<B>) -> Request<A, T>
where A: RequestAdapter {
    let adpt_ = adpt.clone();

    let (tx, rx) = futures::oneshot();

    let exec = Box::new(move || {
        let res = panic::catch_unwind(move || send_request(adpt, builder))

        tx.complete(panic::catch_unwind
    });

    Request {
        adapter: self,
        exec: exec,
        call: call::from_oneshot(rx),
    }
}


fn exec_request<A, B, T>(adpt: &A, builder: RequestBuilder<B>) -> Result<T>
where A: RequestAdapter, B: Body, T: Deserialize {
    adpt.intercept(&mut builder.head);

    let body = builder.body.into_readable()


        let result =
            .send(&adpt.base_url, &adpt.client)
            .and_then(|mut response| try!(adpt.deserializer.deserialize(&mut response)));

    }
}

fn concat_urls(base_url: &Url, path: &Url) {
    base_url.
}


