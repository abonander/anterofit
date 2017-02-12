use hyper::Url;
use hyper::client::{Client, RequestBuilder as NetRequestBuilder};

use parking_lot::{RwLock, RwLockWriteGuard};

use std::sync::Arc;
use std::fmt;

use executor::{DefaultExecutor, Executor, ExecBox};

use mpmc::{self, Sender};

use net::intercept::{Interceptor, Chain, NoIntercept};

use net::request::RequestHead;

use serialize::{Serializer, Deserializer};
use serialize::none::NoSerializer;
use serialize::FromStrDeserializer;

use Result;

/// A builder for `Adapter`. Call `Adapter::builder()` to get an instance.
pub struct AdapterBuilder<S, D, E, I> {
    base_url: Option<Url>,
    client: Option<Client>,
    executor: E,
    interceptor: I,
    serializer: S,
    deserializer: D,
}

impl AdapterBuilder<NoSerializer, FromStrDeserializer, DefaultExecutor, NoIntercept> {
    fn new() -> Self {
        AdapterBuilder {
            base_url: None,
            client: None,
            executor: DefaultExecutor::new,
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
            executor: executor,
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
    pub fn serialize_json(self) -> AdapterBuilder<serialize::json::Serializer, serialize::json::Deserializer, E, I> {
        self.serializer(serialize::json::Serializer)
            .deserializer(serialize::json::Deserializer)
    }
}

impl<S, D, E, I> AdapterBuilder<S, D, E, I>
where S: Serializer, D: Deserializer, E: Executor, I: Interceptor {

    /// Using the supplied types, complete the adapter.
    ///
    /// `<E as Executor>::start()` will be called here.
    pub fn build(self) -> Adapter<S, D> {
        let client_url = ClientUrl {
            base_url: self.base_url,
            client: self.client.unwrap_or_else(Client::new),
        };

        let serialize = Serialize {
            serializer: self.serializer,
            deserializer: self.deserializer,
        };

        Adapter {
            inner: Arc::new(
                Adapter_ {
                    client_url: client_url,
                    serialize: serialize,
                    interceptor: self.interceptor.into_opt_obj(),
                }
            ),
        }
    }
}

/// A shorthand for an adapter with JSON serialization enabled.
#[cfg(any(feature = "rustc-serialize", feature = "serde-json"))]
pub type JsonAdapter= Adapter<::serialize::json::Serializer, ::serialize::json::Deserializer>;

/// The starting point of all Anterofit requests.
///
/// Use `builder()` to start constructing an instance.
pub struct Adapter<S = NoSerializer, D = FromStrDeserializer> {
    inner: Arc<Adapter_<S, D>>,
}

impl<S, D> Clone for Adapter<S, D> {
    fn clone(&self) -> Self {
        Adapter {
            inner: self.inner.clone(),
        }
    }
}

impl Adapter<NoSerializer, FromStrDeserializer> {
    /// Start building an impl of `Adapter` using the default inner types.
    pub fn builder() -> AdapterBuilder<NoSerializer, FromStrDeserializer, DefaultExecutor, NoIntercept> {
        AdapterBuilder::new()
    }
}

impl<S, D> Adapter<S, D> {
    /// Modify this adaptor's interceptor.
    ///
    /// ## Note
    /// Any existing service trait objects and copies of this adapter will be unaffected
    /// by this change.
    pub fn interceptor_mut(&mut self) -> InterceptorMut {
        InterceptorMut(&mut Arc::make_mut(&mut self.inner).interceptor)
    }
}

impl<S, D> fmt::Debug for Adapter<S, D>
where S: fmt::Debug, D: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("anterofit::Adapter")
            .field("base_url", &self.inner.base_url)
            .field("client", &self.inner.client)
            .field("serializer", &self.inner.serializer)
            .field("deserializer", &self.inner.deserializer)
            .field("interceptor", &self.inner.interceptor)
            .finish()
    }
}

/// A mutator for modifying the `Interceptor` of an `Adapter`.
pub struct InterceptorMut<'a>(&'a mut Option<Arc<Interceptor>>);

impl<'a> InterceptorMut<'a> {
    /// Remove the interceptor from the adapter.
    pub fn remove(&mut self) {
        *self.0 = None;
    }

    /// Set a new interceptor, discarding the old one.
    pub fn set<I>(&mut self, new: I) where I: Interceptor {
        *self.0 = new.into_opt_obj();
    }

    /// Chain the given `Interceptor` before the one currently in the adapter.
    ///
    /// Equivalent to `set(before)` if the adapter does not have an interceptor or was constructed
    /// with `NoIntercept` as the interceptor.
    pub fn chain_before<I>(&mut self, before: I) where I: Interceptor {
        *self.0 = match self.0.take() {
            Some(current) => before.chain(current).into_opt_obj(),
            None => before.into_opt_obj(),
        };
    }

    /// Chain the given `Interceptor` after the one currently in the adapter.
    ///
    /// Equivalent to `set(after)` if the adapter does not have an interceptor or was constructed
    /// with `NoIntercept` as the interceptor.
    pub fn chain_after<I>(&mut self, after: I) where I: Interceptor {
        *self.0 = match self.0.take() {
            Some(current) => current.chain(after).into_opt_obj(),
            None => after.into_opt_obj(),
        };
    }

    /// Chain the given `Interceptor`s before and after the one currently in the adapter.
    ///
    /// This saves a level of boxing over calling `chain_before()` and `chain_after()`
    /// separately.
    ///
    /// Equivalent to `set(before.chain(after))` if the adapter does not have an interceptor or
    /// was constructed with `NoIntercept` as the interceptor.
    pub fn chain_around<I1, I2>(&mut self, before: I1, after: I2)
        where I1: Interceptor, I2: Interceptor {
        *self.0 = match self.0.take() {
            Some(current) => before.chain2(current, after).into_opt_obj(),
            None => before.chain(after).into_opt_obj(),
        };
    }
}

struct ClientUrl {
    base_url: Option<Url>,
    client: Client,
}

struct Serialize<S, D> {
    serializer: S,
    deserializer: D,
}

/// Public but not accessible
pub struct Adapter_<S, D> {
    client_url: Arc<ClientUrl>,
    serialize: Arc<Serialize<S, D>>,
    interceptor: Option<Arc<Interceptor>>,
}

impl<S, D> Clone for Adapter_<S, D> {
    fn clone(&self) -> Self {
        Adapter_ {
            client_url: self.client_url.clone(),
            serialize: self.serialize.clone(),
            interceptor: self.interceptor.clone()
        }
    }
}

impl<S, D> Adapter<S, D> {
    pub fn service<S, D, Service: ?Sized>(&self) -> Arc<Service> where Adapter_<S, D>: Service {
        unimplemented!();
    }
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

    /// Enqueue `exec` on this adapter's executor.
    fn enqueue(&self, exec: Box<ExecBox>);

    /// Initialize a `hyper::client::RequestBuilder` from `head`.
    fn request_builder(&self, head: &RequestHead) -> Result<NetRequestBuilder>;
}

impl<S, D> AbsAdapter for Adapter<S, D> where S: Serializer, D: Deserializer {
    type Serializer = S;
    type Deserializer = D;

    fn serializer(&self) -> &S {
        &self.inner.serializer
    }

    fn deserializer(&self) -> &D {
        &self.inner.deserializer
    }
}

impl<S, D> ObjSafeAdapter for Adapter<S, D> where S: Serializer, D: Deserializer {

    fn enqueue(&self, exec: Box<ExecBox>) {
        self.executor.execute(exec)
    }

    fn intercept(&self, head: &mut RequestHead) {
        if let Some(ref interceptor) = self.inner.interceptor {
            interceptor.intercept(head);
        }
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

    fn enqueue(&self, _: Box<ExecBox>) {}

    fn request_builder(&self, _: &RequestHead) -> Result<NetRequestBuilder> {
        unimplemented!()
    }
}