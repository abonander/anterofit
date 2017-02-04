use hyper::Url;
use hyper::client::{Client, RequestBuilder as NetRequestBuilder};

use std::sync::Arc;
use std::fmt;

use executor::{DefaultExecutor, Executor, ExecBox};

use net::intercept::{Interceptor, Chain, NoIntercept};

use net::request::RequestHead;

use serialize::{Serializer, Deserializer};
use serialize::none::NoSerializer;
use serialize::FromStrDeserializer;

use ::Result;

enum Lazy<T> {
    Later(fn () -> T),
    Now(T)
}

impl<T> Lazy<T> {
    fn into_val(self) -> T {
        match self {
            Lazy::Later(init) => init(),
            Lazy::Now(val) => val,
        }
    }
}

/// A builder for `Adapter`. Call `Adapter::builder()` to get an instance.
pub struct AdapterBuilder<S, D, E, I> {
    base_url: Option<Url>,
    client: Option<Client>,
    executor: Lazy<E>,
    interceptor: I,
    serializer: S,
    deserializer: D,
}

impl AdapterBuilder<NoSerializer, FromStrDeserializer, DefaultExecutor, NoIntercept> {
    fn new() -> Self {
        AdapterBuilder {
            base_url: None,
            client: None,
            executor: Lazy::Later(DefaultExecutor::new),
            interceptor: NoIntercept,
            serializer: NoSerializer,
            deserializer: FromStrDeserializer,
        }
    }
}

impl<S, D, E, I> AdapterBuilder<S, D, E, I> {
    /// Set the base URL that the adapter will use for all requests.
    ///
    /// If a base URL is not provided, then all service method URLs are assumed to be absolute.
    pub fn base_url(self, url: Url) -> Self {
        AdapterBuilder { base_url: Some(url), .. self }
    }

    /// Set a `hyper::Client` instance to use with the adapter.
    ///
    /// If not supplied, a default instance will be constructed.
    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Set a new executor for the adapter.
    pub fn executor<E_>(self, executor: E_) -> AdapterBuilder<S, D, E_, I>
        where E: Executor {
        AdapterBuilder {
            base_url: self.base_url,
            client: self.client,
            executor: Lazy::Now(executor),
            interceptor: self.interceptor,
            serializer: self.serializer,
            deserializer: self.deserializer,
        }
    }

    /// Set a new interceptor for the adapter.
    pub fn interceptor<I_>(self, interceptor: I_) -> AdapterBuilder<S, D, E, I_>
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
    pub fn chain_interceptor<I_>(self, next: I_) -> AdapterBuilder<S, D, E, Chain<I, I_>>
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

    /// Set a new `Serializer` impl for the adapter.
    pub fn serializer<S_>(self, serialize: S_) -> AdapterBuilder<S_, D, E, I>
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

    /// Set a new `Deserializer` impl for the adapter.
    pub fn deserializer<D_>(self, deserialize: D_) -> AdapterBuilder<S, D_, E, I>
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
}

#[cfg(any(feature = "rustc-serialize", feature = "serde-json"))]
impl<S, D, E, I> AdapterBuilder<S, D, E, I> {
    /// Convenience method for using JSON serialization.
    ///
    /// Enabled with either the `rust-serialize` feature or the `serde-json` feature.
    pub fn serialize_json(self) -> AdapterBuilder<::serialize::json::Serializer, ::serialize::json::Deserializer, E, I> {
        self.serializer(::serialize::json::Serializer)
            .deserializer(::serialize::json::Deserializer)
    }
}

impl<S, D, E, I> AdapterBuilder<S, D, E, I>
where S: Serializer, D: Deserializer, E: Executor, I: Interceptor {

    /// Using the supplied types, complete the adapter.
    pub fn build(self) -> Adapter<S, D, E> {
        Adapter {
            executor: self.executor.into_val(),
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

/// A shorthand for an adapter with JSON serialization enabled.
#[cfg(any(feature = "rustc-serialize", feature = "serde-json"))]
pub type JsonAdapter<E = DefaultExecutor> = Adapter<::serialize::json::Serializer,
                                                ::serialize::json::Deserializer, E>;

/// The starting point of all Anterofit requests.
///
/// Use `builder()` to start constructing an instance.
pub struct Adapter<S, D, E: ?Sized = DefaultExecutor> {
    inner: Arc<Adapter_<S, D, Interceptor>>,
    executor: E,
}

impl<S, D, E: Clone> Clone for Adapter<S, D, E> {
    fn clone(&self) -> Self {
        Adapter {
            inner: self.inner.clone(),
            executor: self.executor.clone(),
        }
    }
}

impl Adapter<NoSerializer, FromStrDeserializer, DefaultExecutor> {
    /// Start building an impl of `Adapter` using the default inner types.
    pub fn builder() -> AdapterBuilder<NoSerializer, FromStrDeserializer, DefaultExecutor, NoIntercept> {
        AdapterBuilder::new()
    }
}

impl<S, D, E: ?Sized> fmt::Debug for Adapter<S, D, E>
where S: fmt::Debug, D: fmt::Debug, E: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("anterofit::Adapter")
            .field("base_url", &self.inner.base_url)
            .field("client", &self.inner.client)
            .field("serializer", &self.inner.serializer)
            .field("deserializer", &self.inner.deserializer)
            .field("interceptor", &&self.inner.interceptor)
            .field("executor", &&self.executor)
            .finish()
    }
}

struct Adapter_<S, D, I: ?Sized> {
    base_url: Option<Url>,
    client: Client,
    serializer: S,
    deserializer: D,
    // Last field so it may be unsized
    interceptor: I,
}

/// Implemented by `Adapter`. Mainly used to simplify generics.
///
/// Not object-safe.
pub trait AbsAdapter: ObjSafeAdapter + Clone {
    /// The adapter's serializer type.
    type Serializer: Serializer;
    /// The adapter's deserializer type.
    type Deserializer: Deserializer;

    /// Get a reference to the adapter's `Serializer`.
    fn serializer(&self) -> &Self::Serializer;

    /// Get a reference to the adapter's `Deserializer`.
    fn deserializer(&self) -> &Self::Deserializer;
}

/// Object-safe subset of the adapter API.
pub trait ObjSafeAdapter: Send + 'static {
    /// Pass `head` to this adapter's interceptor for modification.
    fn intercept(&self, head: &mut RequestHead);

    /// Execute `exec` on this adapter's executor.
    fn execute(&self, exec: Box<ExecBox>);

    /// Initialize a `hyper::client::RequestBuilder` from `head`.
    fn request_builder(&self, head: &RequestHead) -> Result<NetRequestBuilder>;
}

impl<S, D, E> AbsAdapter for Adapter<S, D, E>
where S: Serializer, D: Deserializer, E: Executor {
    type Serializer = S;
    type Deserializer = D;

    fn serializer(&self) -> &S {
        &self.inner.serializer
    }

    fn deserializer(&self) -> &D {
        &self.inner.deserializer
    }
}

impl<S, D, E> ObjSafeAdapter for Adapter<S, D, E>
where S: Serializer, D: Deserializer, E: Executor {

    fn execute(&self, exec: Box<ExecBox>) {
        self.executor.execute(exec)
    }

    fn intercept(&self, head: &mut RequestHead) {
        self.inner.interceptor.intercept(head);
    }

    fn request_builder(&self, head: &RequestHead) -> Result<NetRequestBuilder> {
        head.init_request(self.inner.base_url.as_ref(), &self.inner.client)
    }
}

/// A `RequestAdapter` with all the methods left unimplemented.
pub const NOOP: &'static ObjSafeAdapter = &NoopAdapter;

struct NoopAdapter;

impl ObjSafeAdapter for NoopAdapter {
    fn intercept(&self, _: &mut RequestHead) {}

    fn execute(&self, _: Box<ExecBox>) {}

    fn request_builder(&self, _: &RequestHead) -> Result<NetRequestBuilder> {
        unimplemented!()
    }
}

#[cfg(feature = "nightly")]
mod nightly {
    use super::Adapter;

    use std::marker::Unsize;
    use std::ops::CoerceUnsized;

    /// Allows `E` to be erased as `Executor`.
    impl<S, D, E: ?Sized, I: ?Sized, E_: ?Sized> CoerceUnsized<Adapter<S, D, E_, I>>
    for Adapter<S, D, E, I> where E: Unsize<E_>, {}

    #[test]
    fn unsize_adapter() {
        use super::Interceptor;

        let _ : Box<Adapter<_, _, Executor>> = Box::new(Adapter::builder().build());
    }
}
