//! Types used to serialize and deserialize request and response bodies, respectively.
//!
//! ## Note
//! If you get an error about duplicate types in this module, make sure you don't have both the
//! `rustc-serialize` and `serde` features enabled

use mime::Mime;

use std::io::{Read, Write};

use ::Result;

pub mod none;

// Until we have a way to describe these features as mutually exclusive, this will
// have to do.
#[cfg(feature = "serde")]
pub mod serde;

#[cfg(feature = "serde")]
pub use self::serde::*;

#[cfg(feature = "rustc-serialize")]
pub mod rustc;

#[cfg(feature = "rustc-serialize")]
pub use self::rustc::*;

// It'd be nice to support both of these at once but unfortunately that's incredibly
// unwieldy without HKT (trust me, I tried).

// Fortunately the traits in both crates have the same signatures (at least for now)
// so they can be used interchangeably.

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
