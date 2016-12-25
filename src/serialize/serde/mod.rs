//! Serialization with [Serde](https://serde.rs)

extern crate serde;

pub use self::serde::{Serialize, Deserialize};

use self::serde::Serializer;

use std::fmt::{Display, Write};

#[cfg(feature = "serde_json")]
pub mod json;

#[cfg(feature = "serde_xml")]
pub mod xml;

/// JSON only allows string keys, so all keys are converted to strings.
impl<K: Display, V: Serialize> Serialize for super::PairMap<K, V> {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error> where S: Serializer {
        let pairs = self.pairs();

        let mut map_s = try!(s.serialize_map(Some(pairs.len())));

        let mut key_buf = String::new();

        for &(ref key, ref val) in pairs {
            key_buf.clear();
            write!(key_buf, "{}", key).expect("Error formatting key");

            try!(s.serialize_map_key(&mut map_s, &key_buf));
            try!(s.serialize_map_value(&mut map_s, val));
        }

        s.serialize_map_end(map_s)
    }
}