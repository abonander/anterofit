use hyper::Url;
use hyper::client::{Client, RequestBuilder as NetRequestBuilder};

use parking_lot::{RwLock, RwLockWriteGuard};

use std::sync::Arc;
use std::fmt;

use executor::{DefaultExecutor, Executor, ExecBox};

use mpmc::{self, Sender};

use net::intercept::{Interceptor, Chain, NoIntercept};

use net::request::RequestHead;

use serialize::{self, Serializer, Deserializer};
use serialize::none::NoSerializer;
use serialize::FromStrDeserializer;

use service::ServiceDelegate;

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
            executor: DefaultExecutor::new(),
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
        let (tx, rx) = mpmc::channel();

        self.executor.start(rx);

        let consts = AdapterConsts {
            base_url: self.base_url,
            client: self.client.unwrap_or_else(Client::new),
            serializer: self.serializer,
            deserializer: self.deserializer,
            sender: tx,
        };

        Adapter {
            inner: Arc::new(
                Adapter_ {
                    consts: Arc::new(consts),
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

impl<S, D> fmt::Debug for Adapter_<S, D>
where S: fmt::Debug, D: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("anterofit::Adapter")
            .field("base_url", &self.consts.base_url)
            .field("client", &self.consts.client)
            .field("serializer", &self.consts.serializer)
            .field("deserializer", &self.consts.deserializer)
            .field("interceptor", &self.interceptor)
            .finish()
    }
}

impl<S, D> fmt::Debug for Adapter<S, D>
where S: fmt::Debug, D: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
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

/// Constant types in an adapter
pub struct AdapterConsts<S, D> {
    pub base_url: Option<Url>,
    pub client: Client,
    pub sender: Sender,
    pub serializer: S,
    pub deserializer: D,
}

/// Public but not accessible
pub struct Adapter_<S, D> {
    consts: Arc<AdapterConsts<S, D>>,
    interceptor: Option<Arc<Interceptor>>,
}

impl<S, D> Clone for Adapter_<S, D> {
    fn clone(&self) -> Self {
        Adapter_ {
            consts: self.consts.clone(),
            interceptor: self.interceptor.clone()
        }
    }
}

impl<S: Serializer, D: Deserializer> Adapter<S, D> {
    pub fn service<Serv: ?Sized>(&self) -> Arc<Serv::Wrapped> where Serv: ServiceDelegate {
        Serv::from_adapter(self.inner.clone())
    }

    pub fn ref_service<Serv: ?Sized>(&self) -> &Serv::Wrapped where Serv: ServiceDelegate {
        Serv::from_ref_adapter(&*self.inner)
    }
}

/// Implemented by private types.
pub trait AbsAdapter: PrivAdapter {}

pub trait PrivAdapter: Send + 'static {
    /// The adapter's serializer type.
    type Ser: Serializer;
    /// The adapter's deserializer type.
    type De: Deserializer;

    fn ref_consts(&self) -> &AdapterConsts<Self::Ser, Self::De>;

    fn consts(&self) -> Arc<AdapterConsts<Self::Ser, Self::De>>;

    fn interceptor(&self) -> Option<Arc<Interceptor>>;
}

impl<S, D> AbsAdapter for Adapter_<S, D> where S: Serializer, D: Deserializer {}

impl<S, D> PrivAdapter for Adapter_<S, D> where S: Serializer, D: Deserializer {
    type Ser = S;
    type De = D;

    fn ref_consts(&self) -> &AdapterConsts<S, D> {
        &self.consts
    }

    fn consts(&self) -> Arc<AdapterConsts<S, D>> {
        self.consts.clone()
    }

    fn interceptor(&self) -> Option<Arc<Interceptor>> {
        self.interceptor.clone()
    }
}
