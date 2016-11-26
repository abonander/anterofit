//! Types used to serialize and deserialize request and response bodies, respectively.
//!
//! ## Note
//! If you get an error about duplicate types or items in this module, make sure you don't have both
//! the `rustc-serialize` and `serde` features enabled.

use mime::Mime;

use std::fmt;
use std::io::{Read, Write};

use ::Result;

pub mod none;

// It'd be nice to support both of these at once but unfortunately that's incredibly
// unwieldy without HKT (trust me, I tried).

// Fortunately the traits in both crates have the same signatures (at least for now)
// so they can be used interchangeably.

// Until we have a way to describe these features as mutually exclusive, this will
// have to do.
#[cfg(feature = "serde")]
pub mod serde;

#[cfg(feature = "serde")]
pub use self::serde::*;

#[cfg(feature = "rustc-serialize")]
pub mod rustc;

#[cfg(feature = "rustc-serialize")]
pub use self::rustc::{
    json,
    Decodable as Deserialize,
    Encodable as Serialize
};

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

/// A simple series of key-value pairs that can be serialized as a map.
///
/// Nothing will be done with duplicate keys.
#[derive(Clone)]
pub struct PairMap<K, V> {
    pairs: Vec<(K, V)>,
}

impl<K, V> PairMap<K, V> {
    /// Create an empty series.
    pub fn new() -> Self {
        PairMap {
            pairs: Vec::new()
        }
    }

    /// Add a key-value pair to the end of this series.
    pub fn insert(&mut self, key: K, val: V) {
        self.pairs.push((key, val));
    }

    /// Get the current series of pairs as a slice.
    pub fn pairs(&self) -> &[(K, V)] {
        &self.pairs
    }

    /// Get the current series of pairs as a mutable reference to a vector.
    pub fn pairs_mut(&mut self) -> &mut Vec<(K, V)> {
        &mut self.pairs
    }

    /// Take the key-value pair series as a vector of 2-tuples.
    pub fn into_pairs(self) -> Vec<(K, V)> {
        self.pairs
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for PairMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut debug = f.debug_map();

        for &(ref key, ref val) in &self.pairs {
            debug.entry(key, val);
        }

        debug.finish()
    }
}

#[test]
fn pair_map_is_serialize() {
    use std::io;

    let pair_map: PairMap<String, String> = PairMap::new();

    let _ = none::NoSerializer.serialize(&pair_map, &mut io::sink());
}