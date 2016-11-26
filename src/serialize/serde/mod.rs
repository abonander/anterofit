//! Serialization with [Serde](https://serde.rs)

extern crate serde;

pub use self::serde::{Serialize, Deserialize};

use self::serde::Serializer;

#[cfg(feature = "serde_json")]
pub mod json;

#[cfg(feature = "serde_xml")]
pub mod xml;

impl<K: Serialize, V: Serialize> Serialize for super::PairMap<K, V> {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error> where S: Serializer {
        let pairs = self.pairs();

        let mut map_s = try!(s.serialize_map(Some(pairs.len())));

        for &(ref key, ref val) in pairs {
            try!(s.serialize_map_key(&mut map_s, key));
            try!(s.serialize_map_value(&mut map_s, val));
        }

        s.serialize_map_end(map_s)
    }
}