//! Serialization with [Serde](https://serde.rs)

extern crate serde;

pub use self::serde::{Serialize, Deserialize};

use self::serde::Serializer;
use self::serde::ser::SerializeMap;
use self::serde::de::Error;

use std::error::Error as StdError;
use std::fmt::{Display, Write};
use std::io::Read;

#[cfg(feature = "serde_json")]
pub mod json;

#[cfg(feature = "serde_xml")]
pub mod xml;

/// JSON only allows string keys, so all keys are converted to strings.
impl<K: Display, V: Serialize> Serialize for super::PairMap<K, V> {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error> where S: Serializer,
                                                                  S::SerializeMap: SerializeMap {
        let pairs = self.pairs();

        let mut map_s = try!(s.serialize_map(Some(pairs.len())));

        let mut key_buf = String::new();

        for &(ref key, ref val) in pairs {
            key_buf.clear();
            write!(key_buf, "{}", key).expect("Error formatting key");

            try!(map_s.serialize_entry(&key_buf, val));
        }

        map_s.end()
    }
}

impl Error for ::Error {
    fn custom<T: Display>(msg: T) -> Self {
        let error: Box<StdError + Send + Sync> = msg.to_string().into();
        ::Error::Deserialize(error)
    }
}

impl super::Deserializer for super::FromStrDeserializer {
    fn deserialize<T: Deserialize, R: Read>(&self, read: &mut R) -> ::Result<T> {
        use self::serde::de::value::ValueDeserializer;

        let mut string = String::new();
        let string = try!(read.read_to_string(&mut string));
        T::deserialize(string.into_deserializer())
    }
}