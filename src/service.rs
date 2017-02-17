use adapter::AbsAdapter;

use serialize::{Serializer, Deserializer};

pub trait ServiceDelegate {
    type Wrapped: ?Sized;

    /// Create an instance of the service trait from the given `Adapter`
    fn from_adapter<A>(adpt: ::std::sync::Arc<A>) -> ::std::sync::Arc<Self::Wrapped>
    where A: AbsAdapter;

    /// Create an instance of the service trait from the given `Adapter`
    fn from_ref_adapter<A>(adpt: &A) -> &Self::Wrapped where A: AbsAdapter;
}