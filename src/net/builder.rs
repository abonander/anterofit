
use hyper::client::IntoUrl;
use hyper::header::{Headers, Header, HeaderFormat};
use hyper::method::Method;
use hyper::Url;

use multipart::client::lazy::Multipart;

use std::fmt::{self, Write};
use std::io::{self, Empty, Read};
use std::mem;

use super::Body;

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



