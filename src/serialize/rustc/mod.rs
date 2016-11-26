//! Serialization with [`rustc-serialize`](https://github.com/rust-lang-nursery/rustc-serialize).

pub use rustc_serialize::{
    Decodable,
    Encodable,
    Decoder,
    Encoder
};

pub mod json;

impl<K: Encodable, V: Encodable> Encodable for super::PairMap<K, V> {
    fn encode<E: Encoder>(&self, en: &mut E) -> Result<(), E::Error> {
        let pairs = self.pairs();

        en.emit_map(pairs.len(), |en| {
            for (idx, &(ref key, ref val)) in pairs.iter().enumerate() {
                try!(en.emit_map_elt_key(idx, |en| key.encode(en)));
                try!(en.emit_map_elt_val(idx, |en| val.encode(en)));
            }

            Ok(())
        })
    }
}