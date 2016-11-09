use hyper::Url;
use hyper::client::{Client, RequestBuilder as NetRequestBuilder};

use std::error::Error;
use std::io::{Read, Write};
use std::panic::UnwindSafe;
use std::sync::Arc;

use executor::{DefaultExecutor, Executor, ExecBox};

use net::body::Body;

use net::intercept::{Interceptor, Chain};

use net::request::{self, Request, RequestHead, RequestBuilder};

use serialize::{Serializer, Deserializer, NoSerializer, NoDeserializer, Serialize, Deserialize};

use ::Result;

pub struct AdapterBuilder<E, I, S, D> {
    base_url: Url,
    client: Option<Client>,
    executor: E,
    interceptor: I,
    serializer: S,
    deserializer: D,
}

impl AdapterBuilder<DefaultExecutor, (), NoSerializer, NoDeserializer> {
    fn new(url: Url) -> Self {
        AdapterBuilder {
            base_url: url,
            client: None,
            executor: DefaultExecutor::new(),
            interceptor: (),
            serializer: NoSerializer,
            deserializer: NoDeserializer,
        }
    }
}

impl<E, I, S, D> AdapterBuilder<E, I, S, D> {
    pub fn interceptor<I_>(self, interceptor: I_) -> AdapterBuilder<E, I_, S, D>
    where I_: Interceptor {
        AdapterBuilder {
            base_url: self.base_url,
            client: self.client,
            executor: self.executor,
            interceptor: interceptor,
            serializer: self.serializer,
            deserializer: self.deserializer,
        }
    }

    pub fn chain_interceptor<I_>(self, next: I_) -> AdapterBuilder<E, Chain<I, I_>, S, D>
    where I_: Interceptor {
        AdapterBuilder {
            base_url: self.base_url,
            client: self.client,
            executor: self.executor,
            interceptor: Chain::new(self.interceptor, next),
            serializer: self.serializer,
            deserializer: self.deserializer,
        }
    }

    pub fn executor<E_>(self, executor: E_) -> AdapterBuilder<E_, I, S, D>
    where E: Executor {
        AdapterBuilder {
            base_url: self.base_url,
            client: self.client,
            executor: executor,
            interceptor: self.interceptor,
            serializer: self.serializer,
            deserializer: self.deserializer,
        }
    }

    pub fn serialize<S_>(self, serialize: S_) -> AdapterBuilder<E, I, S_, D>
    where S_: Serializer {
        AdapterBuilder {
            base_url: self.base_url,
            client: self.client,
            executor: self.executor,
            interceptor: self.interceptor,
            serializer: serialize,
            deserializer: self.deserializer,
        }
    }

    pub fn deserialize<D_>(self, deserialize: D_) -> AdapterBuilder<E, I, S, D_>
    where D_: Deserializer {
        AdapterBuilder {
            base_url: self.base_url,
            client: self.client,
            executor: self.executor,
            interceptor: self.interceptor,
            serializer: self.serializer,
            deserializer: deserialize,
        }
    }
    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }
}

impl<E, I, S, D> AdapterBuilder<E, I, S, D>
where E: Executor, I: Interceptor, S: Serializer, D: Deserializer {
    pub fn build(self) -> Adapter<E, I, S, D> {
        Adapter {
            executor: self.executor,
            inner: Arc::new(
                Adapter_ {
                    base_url: self.base_url,
                    client: self.client.unwrap_or_else(Client::new),
                    interceptor: self.interceptor,
                    serializer: self.serializer,
                    deserializer: self.deserializer
                }
            )
        }
    }
}

#[derive(Debug)]
pub struct Adapter<E, I, S, D> {
    executor: E,
    inner: Arc<Adapter_<I, S, D>>,
}

impl Adapter<DefaultExecutor, (), NoSerializer, NoDeserializer> {
    pub fn builder(url: Url) -> AdapterBuilder<DefaultExecutor, (), NoSerializer, NoDeserializer> {
        AdapterBuilder::new(url)
    }
}

impl<E: Clone, I, S, D> Clone for Adapter<E, I, S, D> {
    fn clone(&self) -> Self {
        Adapter {
            executor: self.executor.clone(),
            inner: self.inner.clone(),
        }
    }
}

#[derive(Debug)]
struct Adapter_<I, S, D> {
    base_url: Url,
    client: Client,
    interceptor: I,
    serializer: S,
    deserializer: D,
}

pub trait RequestAdapter: RequestAdapter_ {
    fn request<B, T>(&self, builder: RequestBuilder<B>) -> Request<Self, T>
        where B: Body, T: Deserialize + Send + 'static;
}

impl<A> RequestAdapter for A where A: RequestAdapter_ {
    fn request<B, T>(&self, builder: RequestBuilder<B>) -> Request<Self, T>
        where B: Body, T: Deserialize + Send + 'static {
        request::new(self, builder)
    }
}

pub trait RequestAdapter_: Send + Clone + 'static {
    fn intercept(&self, head: &mut RequestHead);

    fn execute(&self, exec: Box<ExecBox>);

    fn serialize<T: Serialize, W: Write>(&self, val: &T, to: &mut W) -> Result<()>;

    fn deserialize<T: Deserialize, R: Read>(&self, from: &mut R) -> Result<T>;

    fn request_builder(&self, head: RequestHead) -> Result<NetRequestBuilder>;
}

impl<E, I, S, D> RequestAdapter_ for Adapter<E, I, S, D>
where E: Executor, I: Interceptor, S: Serializer, D: Deserializer {

    fn execute(&self, exec: Box<ExecBox>) {
        self.executor.execute(exec)
    }

    fn serialize<T: Serialize, W: Write>(&self, val: &T, to: &mut W) -> Result<()> {
        self.inner.serializer.serialize(val, to)
    }

    fn deserialize<T: Deserialize, R: Read>(&self, from: &mut R) -> Result<T> {
        self.inner.deserializer.deserialize(from)
    }

    fn intercept(&self, head: &mut RequestHead) {
        self.inner.interceptor.intercept(head);
    }

    fn request_builder(&self, head: RequestHead) -> Result<NetRequestBuilder> {
        head.init_request(&self.inner.base_url, &self.inner.client)
    }
}