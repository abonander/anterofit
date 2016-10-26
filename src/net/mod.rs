use hyper::{Client, Url};
use hyper::client::{IntoUrl};
pub use hyper::method::Method;

pub use hyper::header::Headers;

use serialize::{Serializer, Deserializer, NoSerializer, NoDeserializer};

pub use self::intercept::{Interceptor, Chain};

pub use self::body::{Fields, Body};

#[doc(noinline)]
pub use self::body::{FileField, AddField};

pub use self::builder::{RequestHead, RequestBuilder};

mod intercept;

mod body;

mod builder;

use std::sync::Arc;

pub struct AdapterBuilder<I, S, D> {
    base_url: Url,
    client: Option<Client>,
    interceptor: I,
    serializer: S,
    deserializer: D,
}

impl AdapterBuilder<(), NoSerializer, NoDeserializer> {
    fn new(url: Url) -> Self {
        AdapterBuilder {
            base_url: url,
            client: None,
            interceptor: (),
            serializer: NoSerializer,
            deserializer: NoDeserializer,
        }
    }
}

impl<I, S, D> AdapterBuilder<I, S, D> {
    pub fn interceptor<I_>(self, interceptor: I_) -> AdapterBuilder<I_, S, D> {
        AdapterBuilder {
            base_url: self.base_url,
            client: self.client,
            interceptor: interceptor,
            serializer: self.serializer,
            deserializer: self.deserializer,
        }
    }

    pub fn chain_interceptor<I_>(self, next: I_) -> AdapterBuilder<Chain<I, I_>, S, D> {
        AdapterBuilder {
            base_url: self.base_url,
            client: self.client,
            interceptor: Chain::new(self.interceptor, next),
            serializer: self.serializer,
            deserializer: self.deserializer,
        }
    }

    pub fn serialize<S_>(self, serialize: S_) -> AdapterBuilder<I, S_, D> {
        AdapterBuilder {
            base_url: self.base_url,
            client: self.client,
            interceptor: self.interceptor,
            serializer: serialize,
            deserializer: self.deserializer,
        }
    }

    pub fn deserialize<D_>(self, deserialize: D_) -> AdapterBuilder<I, S, D_> {
        AdapterBuilder {
            base_url: self.base_url,
            client: self.client,
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

impl<I, S, D> AdapterBuilder<I, S, D> where I: Interceptor, S: Serializer, D: Deserializer {
    pub fn build(self) -> Adapter<I, S, D> {
        Adapter(Arc::new(
            Adapter_ {
            base_url: self.base_url,
            client: self.client.unwrap_or_else(Client::new),
            interceptor: self.interceptor,
            serialize: self.serializer,
            deserialize: self.deserializer
        }
        ))
    }
}

pub struct Adapter<I, S, D>(Arc<Adapter_<I, S, D>>);

impl<I, S, D> Adapter<I, S, D> {
    pub fn builder(url: Url) -> AdapterBuilder<(), NoSerializer, NoDeserializer> {
        AdapterBuilder::new(url)
    }
}

struct Adapter_<I, S, D> {
    base_url: Url,
    client: Client,
    interceptor: I,
    serialize: S,
    deserialize: D,
}

pub trait RequestAdapter {}
