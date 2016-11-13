pub use serde::{Serialize, Deserialize};

use mime::Mime;

use std::io::{Read, Write};

use ::Result;

mod none;

pub use self::none::*;

/// A trait describing types which can concurrently serialize other types into byte-streams.
pub trait Serializer: Send + Sync + 'static {
    /// Serialize `T` to `write`, returning any errors.
    fn serialize<T: Serialize, W: Write>(&self, val: &T, write: &mut W) -> Result<()>;

    /// Return the MIME type of the serialized content, if applicable.
    ///
    /// Used to set the `Content-Type` header of the request this serializer
    /// is being used for.
    fn content_type(&self) -> Option<Mime>;
}

/// A trait describing types which can concurrently deserialize other types from byte-streams.
pub trait Deserializer: Send + Sync + 'static {
    /// Deserialize `T` from `read`, returning the result.
    fn deserialize<T: Deserialize, R: Read>(&self, read: &mut R) -> Result<T>;
}

macro_rules! modules {
    ($($name:ident = $strname:expr),*) => (
        $(
            #[cfg(feature = $strname)]
            pub mod $name;

            #[cfg(not(feature = $strname))]
            pub mod $name {
                /// Empty error type to fill the associated variant of `error::Error`.
                quick_error! {
                    #[derive(Debug)]
                    pub enum Error {}
                }
            }
        )*
    )
}

modules! {
    json = "json", xml = "xml"
}
