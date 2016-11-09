use hyper::{Client, Url, RequestAdapter as NetRequestAdapter};
use hyper::client::{IntoUrl};

pub use hyper::method::Method;

pub use hyper::header::Headers;

use std::error::Error;
use std::panic::UnwindSafe;
use std::sync::Arc;

use serialize::{Serializer, Deserializer, NoSerializer, NoDeserializer};

pub use self::intercept::{Interceptor, Chain};

pub use self::body::*;

pub use call::Call;

pub use self::executor::*;

pub use self::request::{RequestHead, RequestBuilder, Request};

mod body;

mod call;

mod intercept;

mod request;

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
    pub fn interceptor<I_>(self, interceptor: I_) -> AdapterBuilder<E, I_, S, D> {
        AdapterBuilder {
            base_url: self.base_url,
            client: self.client,
            executor: self.executor,
            interceptor: interceptor,
            serializer: self.serializer,
            deserializer: self.deserializer,
        }
    }

    pub fn chain_interceptor<I_>(self, next: I_) -> AdapterBuilder<E, Chain<I, I_>, S, D> {
        AdapterBuilder {
            base_url: self.base_url,
            client: self.client,
            executor: self.executor,
            interceptor: Chain::new(self.interceptor, next),
            serializer: self.serializer,
            deserializer: self.deserializer,
        }
    }

    pub fn executor<E_>(self, executor: E_) -> AdapterBuilder<E_, I, S, D> {
        AdapterBuilder {
            base_url: self.base_url,
            client: self.client,
            executor: executor,
            interceptor: self.interceptor,
            serializer: self.serializer,
            deserializer: self.deserializer,
        }
    }

    pub fn serialize<S_>(self, serialize: S_) -> AdapterBuilder<E, I, S_, D> {
        AdapterBuilder {
            base_url: self.base_url,
            client: self.client,
            executor: self.executor,
            interceptor: self.interceptor,
            serializer: serialize,
            deserializer: self.deserializer,
        }
    }

    pub fn deserialize<D_>(self, deserialize: D_) -> AdapterBuilder<E, I, S, D_> {
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
        Adapter(Arc::new(
            Adapter_ {
                base_url: self.base_url,
                client: self.client.unwrap_or_else(Client::new),
                executor: self.executor,
                interceptor: self.interceptor,
                serialize: self.serializer,
                deserialize: self.deserializer
            }
        ))
    }
}

pub struct Adapter<E, I, S, D>(Arc<Adapter_<E, I, S, D>>);

impl<E, I, S, D> Adapter<E, I, S, D> {
    pub fn builder(url: Url) -> AdapterBuilder<DefaultExecutor, (), NoSerializer, NoDeserializer> {
        AdapterBuilder::new(url)
    }
}

impl<E, I, S, D> Clone for Adapter<E, I, S, D> {
    fn clone(&self) -> Self {
        Adapter(self.0.clone())
    }
}

struct Adapter_<E, I, S, D> {
    base_url: Url,
    client: Client,
    executor: E,
    interceptor: I,
    serialize: S,
    deserialize: D,
}

pub trait RequestAdapter: RequestAdapter_ {
    fn request<B, T>(&self, builder: RequestBuilder<B>) -> Request<Self, T>
    where B: Body, T: Deserialize;
}

impl<T> RequestAdapter for T where T: RequestAdapter_ {
    fn request<B, T>(&self, builder: RequestBuilder<B>) -> Request<Self, T> where B: Body, T: Deserialize {
        request::new(self, builder)
    }
}

trait RequestAdapter_: Send + Clone + 'static + UnwindSafe {
    fn intercept(&self, head: &mut RequestHead);


    fn execute(&self, exec: Box<ExecBox>);

    fn serialize<T: Serialize, W: Write>(&self, val: T, to: &mut W) -> Result<()>;

    fn deserialize<T: Deserialize, R: Read>(&self, from: &mut R) -> Result<T>;

    fn request_builder(&self, head: RequestHead) -> Result<NetRequestBuilder>;
}

impl<E, I, S, D> RequestAdapter_ for Adapter<E, I, S, D>
where E: Executor, I: Interceptor, S: Serializer, D: Deserializer {

    fn execute(&self, exec: Box<ExecBox>) {
        self.0.executor.execute(exec)
    }

    fn serialize<T: Serialize, W: Write>(&self, val: T, to: &mut W) -> Result<()> {
        try!(self.0.serializer.serialize(val, to))
    }

    fn deserialize<T: Deserialize, R: Read>(&self, from: &mut R) -> Result<T> {
        try!(self.0.deserialize.deserialize(from))
    }

    fn intercept(&self, head: &mut RequestHead) {
        self.0.interceptor.intercept(head);
    }

    fn request_builder(&self, head: RequestHead) -> Result<NetRequestBuilder> {
        head.init_request(&self.0.base_url, &self.0.client)
    }
}
