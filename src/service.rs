use adapter::AbsAdapter;

use serialize::{Serializer, Deserializer};

pub trait ServiceDelegate<S: Serializer, D: Deserializer> {
    type Wrapped: AbsService<Ser=S, De=D> + ?Sized;

    /// Create an instance of the service trait from the given `Adapter`
    fn from_adapter<A>(adpt: ::std::sync::Arc<A>) -> ::std::sync::Arc<Self::Wrapped>
    where A: AbsAdapter<Ser=S, De=D>;

    /// Create an instance of the service trait from the given `Adapter`
    fn from_ref_adapter<A>(adpt: &A) -> &Self::Wrapped where A: AbsAdapter<Ser=S, De=D>;
}

pub trait AbsService: AbsAdapter {
    type Ser: Serializer;
    type De: Deserializer;
}

impl<A> AbsService for A where A: AbsAdapter {}