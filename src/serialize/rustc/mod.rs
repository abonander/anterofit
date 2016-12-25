//! Serialization with [`rustc-serialize`](https://github.com/rust-lang-nursery/rustc-serialize).

pub use rustc_serialize::{
    Decodable,
    Encodable,
    Decoder,
    Encoder
};

use std::fmt::{Display, Write};

pub mod json;

/// JSON only allows string keys, so all keys are converted to strings.
impl<K: Display, V: Encodable> Encodable for super::PairMap<K, V> {
    fn encode<E: Encoder>(&self, en: &mut E) -> Result<(), E::Error> {
        let pairs = self.pairs();

        en.emit_map(pairs.len(), |en| {
            let mut key_buf = String::new();

            for (idx, &(ref key, ref val)) in pairs.iter().enumerate() {
                key_buf.clear();
                write!(key_buf, "{}", key).expect("Error formatting key");

                try!(en.emit_map_elt_key(idx, |en| key_buf.encode(en)));
                try!(en.emit_map_elt_val(idx, |en| val.encode(en)));
            }

            Ok(())
        })
    }
}