use hyper::Url;
use hyper::client::{Client, RequestBuilder as NetRequestBuilder};

use std::io::{Read, Write};
use std::sync::Arc;

use executor::{DefaultExecutor, Executor, ExecBox};

use mime::Mime;

use net::body::Body;

use net::intercept::{Interceptor, NoIntercept, Chain};

use net::request::{Request, RequestHead, RequestBuilder};

use net::response::FromResponse;

use serialize::{Serializer, Deserializer, NoSerializer, NoDeserializer, Serialize, Deserialize};

use ::Result;

/// A builder for `Adapter`. Call `Adapter::builder()` to get an instance.
pub struct AdapterBuilder<E, I, S, D> {
    base_url: Option<Url>,
    client: Option<Client>,
    executor: E,
    interceptor: I,
    serializer: S,
    deserializer: D,
}

impl AdapterBuilder<DefaultExecutor, NoIntercept, NoSerializer, NoDeserializer> {
    fn new() -> Self {
        AdapterBuilder {
            base_url: None,
            client: None,
            executor: DefaultExecutor::new(),
            interceptor: NoIntercept,
            serializer: NoSerializer,
            deserializer: NoDeserializer,
        }
    }
}

impl<E, I, S, D> AdapterBuilder<E, I, S, D> {
    /// Set the base URL that this adapter will use for all requests.
    ///
    /// If a base URL is not provided, then all service method URLs are assumed to be absolute.
    pub fn base_url(self, url: Url) -> Self {
        AdapterBuilder { base_url: Some(url), .. self }
    }

    /// Set a new interceptor for the adapter.
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

    /// Chain a new interceptor with the current one. They will be called in-order.
    pub fn chain_interceptor<I_>(self, next: I_) -> AdapterBuilder<E, Chain<I, I_>, S, D>
    where I: Interceptor, I_: Interceptor {
        AdapterBuilder {
            base_url: self.base_url,
            client: self.client,
            executor: self.executor,
            interceptor: self.interceptor.chain(next),
            serializer: self.serializer,
            deserializer: self.deserializer,
        }
    }

    /// Set a new executor for the adapter.
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

    /// Set a new `Serializer` impl for this adapter.
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

    /// Set a new `Deserializer` impl for this adapter.
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

    /// Set a `hyper::Client` instance to use with this adapter.
    ///
    /// If not supplied, a default instance will be constructed.
    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }
}

impl<E, I, S, D> AdapterBuilder<E, I, S, D>
where E: Executor, I: Interceptor, S: Serializer, D: Deserializer {

    /// Using the supplied types, complete the adapter.
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

/// The starting point of all Anterofit requests.
///
/// Use `builder()` to start constructing an instance.
#[derive(Debug)]
pub struct Adapter<E, I, S, D> {
    executor: E,
    inner: Arc<Adapter_<I, S, D>>,
}

impl Adapter<DefaultExecutor, NoIntercept, NoSerializer, NoDeserializer> {
    /// Start building an instance of `Adapter` using the default inner types.
    pub fn builder() -> AdapterBuilder<DefaultExecutor, NoIntercept, NoSerializer, NoDeserializer> {
        AdapterBuilder::new()
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
    base_url: Option<Url>,
    client: Client,
    interceptor: I,
    serializer: S,
    deserializer: D,
}

/// A trait describing an adapter which can be used to execute a request.
///
/// Mainly used to simplify generics.
pub trait RequestAdapter: Send + Clone + 'static {
    /// Create a new `Request` using `builder`.
    fn request<B, T>(&self, builder: RequestBuilder<B>) -> Request<Self, T>
        where B: Body, T: FromResponse;

    /// Pass `head` to this adapter's interceptor for modification.
    fn intercept(&self, head: &mut RequestHead);

    /// Execute `exec` on this adapter's executor.
    fn execute(&self, exec: Box<ExecBox>);

    /// Use this adapter's `Serializer` to serialize `val` into `to`.
    fn serialize<T: Serialize, W: Write>(&self, val: &T, to: &mut W) -> Result<()>;

    /// Return the MIME type for this adapter's serializer, if applicable.
    fn serializer_content_type(&self) -> Option<Mime>;

    /// Use this adapter's `Deserializer` to read `T` from `from`.
    fn deserialize<T: Deserialize, R: Read>(&self, from: &mut R) -> Result<T>;

    /// Initialize a `hyper::client::RequestBuilder` from `head`.
    fn request_builder(&self, head: RequestHead) -> Result<NetRequestBuilder>;
}

impl<E, I, S, D> RequestAdapter for Adapter<E, I, S, D>
where E: Executor, I: Interceptor, S: Serializer, D: Deserializer {
    fn request<B, T>(&self, builder: RequestBuilder<B>) -> Request<Self, T>
        where B: Body, T: FromResponse {

        Request::ready(self, builder)
    }

    fn execute(&self, exec: Box<ExecBox>) {
        self.executor.execute(exec)
    }

    fn serialize<T: Serialize, W: Write>(&self, val: &T, to: &mut W) -> Result<()> {
        self.inner.serializer.serialize(val, to)
    }

    fn serializer_content_type(&self) -> Option<Mime> {
        self.inner.serializer.content_type()
    }

    fn deserialize<T: Deserialize, R: Read>(&self, from: &mut R) -> Result<T> {
        self.inner.deserializer.deserialize(from)
    }

    fn intercept(&self, head: &mut RequestHead) {
        self.inner.interceptor.intercept(head);
    }

    fn request_builder(&self, head: RequestHead) -> Result<NetRequestBuilder> {
        head.init_request(self.inner.base_url.as_ref(), &self.inner.client)
    }
}