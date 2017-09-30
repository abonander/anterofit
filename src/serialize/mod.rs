//! Types used to serialize and deserialize request and response bodies, respectively.
//!
//! ## Note
//! If you get an error about duplicate types or items in this module, make sure you don't have both
//! the `rustc-serialize` and `serde` features enabled.

use mime::Mime;

use std::fmt::{self, Display};
use std::io::{Read, Write};

pub mod none;

pub use serde::Serialize;
pub use serde::de::DeserializeOwned;
use serde;
use serde::ser::SerializeMap;

use std::error::Error as StdError;

#[cfg(feature = "serde_json")]
pub mod json;

#[cfg(feature = "serde_xml")]
pub mod xml;

/// JSON only allows string keys, so all keys are converted to strings.
impl<K: Display, V: Serialize> Serialize for PairMap<K, V> {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error> where S: ::serde::Serializer,
                                                                  S::SerializeMap: SerializeMap {
        use std::fmt::Write;

        let pairs = self.pairs();

        let mut map_s = try!(s.serialize_map(Some(pairs.len())));

        let mut key_buf = String::new();

        for &(ref key, ref val) in pairs {
            key_buf.clear();
            write!(key_buf, "{}", key).expect("Error formatting key");

            map_s.serialize_entry(&key_buf, val)?;
        }

        map_s.end()
    }
}

impl serde::ser::Error for ::Error {
    fn custom<T: Display>(msg: T) -> Self {
        let error: Box<StdError + Send + Sync> = msg.to_string().into();
        ::Error::Deserialize(error)
    }
}

impl serde::de::Error for ::Error {
    fn custom<T: Display>(msg: T) -> Self {
        let error: Box<StdError + Send + Sync> = msg.to_string().into();
        ::Error::Deserialize(error)
    }
}

/// A trait describing types which can concurrently serialize other types into byte-streams.
pub trait Serializer: Send + Sync + 'static {
    /// Serialize `T` to `write`, returning any errors.
    fn serialize<T: Serialize, W: Write>(&self, val: &T, write: &mut W) -> ::Result<()>;

    /// Return the MIME type of the serialized content, if applicable.
    ///
    /// Used to set the `Content-Type` header of the request this serializer
    /// is being used for.
    fn content_type(&self) -> Option<Mime>;
}

/// A trait describing types which can concurrently deserialize other types from byte-streams.
pub trait Deserializer: Send + Sync + 'static {
    /// Deserialize `T` from `read`, returning the result.
    fn deserialize<T: DeserializeOwned, R: Read>(&self, read: &mut R) -> ::Result<T>;
}

/// A deserializer which attempts to parse values from the response as a string.
pub struct FromStrDeserializer;

impl Deserializer for FromStrDeserializer {
    fn deserialize<T: DeserializeOwned, R: Read>(&self, read: &mut R) -> ::Result<T> {
        use serde::de::IntoDeserializer;

        let mut string = String::new();
        let string = read.read_to_string(&mut string)?;
        T::deserialize(string.into_deserializer())
    }
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
