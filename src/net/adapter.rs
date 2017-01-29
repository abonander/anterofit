use hyper::Url;
use hyper::client::{Client, RequestBuilder as NetRequestBuilder};

use std::sync::Arc;

use executor::{DefaultExecutor, Executor, ExecBox};

use net::intercept::{Interceptor, Chain, NoIntercept};

use net::request::RequestHead;

use serialize::{Serializer, Deserializer};
use serialize::none::NoSerializer;
use serialize::FromStrDeserializer;

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

impl AdapterBuilder<DefaultExecutor, NoIntercept, NoSerializer, FromStrDeserializer> {
    fn new() -> Self {
        AdapterBuilder {
            base_url: None,
            client: None,
            executor: DefaultExecutor::new(),
            interceptor: NoIntercept,
            serializer: NoSerializer,
            deserializer: FromStrDeserializer,
        }
    }
}

impl<E, I, S, D> AdapterBuilder<E, I, S, D> {
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

    /// Box the adapter's `Interceptor`.
    pub fn box_interceptor(self) -> AdapterBuilder<E, Box<Interceptor>, S, D>
    where I: Interceptor {
        // Necessary to force coercion to trait object
        let boxed: Box<Interceptor> = Box::new(self.interceptor);

        AdapterBuilder {
            base_url: self.base_url,
            client: self.client,
            executor: self.executor,
            interceptor: boxed,
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

    /// Set a new `Serializer` impl for the adapter.
    pub fn serializer<S_>(self, serialize: S_) -> AdapterBuilder<E, I, S_, D>
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
    pub fn deserializer<D_>(self, deserialize: D_) -> AdapterBuilder<E, I, S, D_>
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
impl<E, I, S, D> AdapterBuilder<E, I, S, D> {
    /// Convenience method for using JSON serialization.
    ///
    /// Enabled with either the `rust-serialize` feature or the `serde-json` feature.
    pub fn serialize_json(self) -> AdapterBuilder<E, I, ::serialize::json::Serializer, ::serialize::json::Deserializer> {
        self.serializer(::serialize::json::Serializer)
            .deserializer(::serialize::json::Deserializer)
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

/// A shorthand for an adapter with JSON serialization enabled.
#[cfg(any(feature = "rustc-serialize", feature = "serde-json"))]
pub type JsonAdapter<E = DefaultExecutor,
                     I = NoIntercept> = Adapter<E, I, ::serialize::json::Serializer,
                                                ::serialize::json::Deserializer>;

/// The starting point of all Anterofit requests.
///
/// Use `builder()` to start constructing an instance.
#[derive(Debug)]
pub struct Adapter<E, I: ?Sized, S, D> {
    executor: E,
    inner: Arc<Adapter_<I, S, D>>,
}

impl<E: Clone, I: ?Sized, S, D> Clone for Adapter<E, I, S, D> {
    fn clone(&self) -> Self {
        Adapter {
            executor: self.executor.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl<E: Clone, I: Interceptor, S, D> Adapter<E, I, S, D> {
    /// Type-erase the adaptor's `Interceptor`.
    ///
    /// Useful for when you want to be able to name the `Adapter` type
    /// but you're using a closure or a long interceptor chain.
    ///
    /// With the `nightly` feature, unsizing coercion is implemented, so you don't need
    /// to call this method explicitly to get this effect.
    pub fn erase(self) -> Adapter<E, Interceptor, S, D> {
        let inner: Arc<Adapter_<Interceptor, S, D>> = self.inner;

        Adapter {
            executor: self.executor,
            inner: inner,
        }
    }
}

impl Adapter<DefaultExecutor, NoIntercept, NoSerializer, FromStrDeserializer> {
    /// Start building an impl of `Adapter` using the default inner types.
    pub fn builder() -> AdapterBuilder<DefaultExecutor, NoIntercept, NoSerializer, FromStrDeserializer> {
        AdapterBuilder::new()
    }
}

#[derive(Debug)]
struct Adapter_<I: ?Sized, S, D> {
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

impl<E, I: ?Sized, S, D> AbsAdapter for Adapter<E, I, S, D>
where E: Executor, I: Interceptor, S: Serializer, D: Deserializer {
    type Serializer = S;
    type Deserializer = D;

    fn serializer(&self) -> &S {
        &self.inner.serializer
    }

    fn deserializer(&self) -> &D {
        &self.inner.deserializer
    }
}

impl<E, I: ?Sized, S, D> ObjSafeAdapter for Adapter<E, I, S, D>
where E: Executor, I: Interceptor, S: Serializer, D: Deserializer {

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

    /// Allows `I` to be erased as `Interceptor`.
    impl<E, I, I_, S, D> CoerceUnsized<Adapter<E, I_, S, D>>
    for Adapter<E, I, S, D> where I: Unsize<I_> + ?Sized, I_: ?Sized {}

    #[test]
    fn unsize_adapter() {
        use super::Interceptor;

        let _ : Adapter<_, Interceptor, _, _> = Adapter::builder().build();
    }

}
