//! Serialization with [`rustc-serialize`](https://github.com/rust-lang-nursery/rustc-serialize).

pub use rustc_serialize::{
    Decodable,
    Encodable,
    Decoder,
    Encoder
};

use std::error::Error;
use std::fmt::{Display, Write};
use std::io::Read;
use std::str::FromStr;

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
/// ### Feature: `rustc-serialize`
///
/// Uses `std::str::FromStr` to parse values from response string.
///
/// ## Errors
/// For complex types which expect a full `Decoder` impl.
impl super::Deserializer for super::FromStrDeserializer {
    fn deserialize<T: super::Deserialize, R: Read>(&self, read: &mut R) -> ::Result<T> {
        T::decode(&mut FromStrImpl(read))
    }
}

struct FromStrImpl<R>(R);

impl<R: Read> FromStrImpl<R> {
    fn read_string(&mut self) -> ::Result<String> {
        let mut string = String::new();
        let _ = try!(self.0.read_to_string(&mut string));
        Ok(string)
    }

    fn read_val<T: FromStr>(&mut self) -> ::Result<T> where <T as FromStr>::Err: Error + Send + 'static {
        self.read_string().and_then(|s| ::Error::map_deserialize(s.parse()))
    }

    fn read_char(&mut self) -> ::Result<char> {
        let string = try!(self.read_string());
        string.chars().next().ok_or_else(|| ::Error::deserialize("Unexpected end of input"))
    }
}

macro_rules! unsupported (
    ($method:ident) => (
        Err(::Error::deserialize(concat!("`rustc_serialize::Decoder::", stringify!($method),
                                     "()` is not supported by `FromStrDeserializer`")))
    )
);

#[allow(unused_variables)]
impl<R: Read> Decoder for FromStrImpl<R> {
    type Error = ::Error;

    fn read_nil(&mut self) -> Result<(), Self::Error> {
        let _ = try!(self.read_string());
        Ok(())
    }

    fn read_usize(&mut self) -> Result<usize, Self::Error> {
        self.read_val()
    }

    fn read_u64(&mut self) -> Result<u64, Self::Error> {
        self.read_val()
    }

    fn read_u32(&mut self) -> Result<u32, Self::Error> {
        self.read_val()
    }

    fn read_u16(&mut self) -> Result<u16, Self::Error> {
        self.read_val()
    }

    fn read_u8(&mut self) -> Result<u8, Self::Error> {
        self.read_val()
    }

    fn read_isize(&mut self) -> Result<isize, Self::Error> {
        self.read_val()
    }

    fn read_i64(&mut self) -> Result<i64, Self::Error> {
        self.read_val()
    }

    fn read_i32(&mut self) -> Result<i32, Self::Error> {
        self.read_val()
    }

    fn read_i16(&mut self) -> Result<i16, Self::Error> {
        self.read_val()
    }

    fn read_i8(&mut self) -> Result<i8, Self::Error> {
        self.read_val()
    }

    fn read_bool(&mut self) -> Result<bool, Self::Error> {
        self.read_val()
    }

    fn read_f64(&mut self) -> Result<f64, Self::Error> {
        self.read_val()
    }

    fn read_f32(&mut self) -> Result<f32, Self::Error> {
        self.read_val()
    }

    fn read_char(&mut self) -> Result<char, Self::Error> {
        self.read_char()
    }

    fn read_str(&mut self) -> Result<String, Self::Error> {
        self.read_val()
    }

    fn read_enum<T, F>(&mut self, name: &str, f: F) -> Result<T, Self::Error> where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        unsupported!(read_enum)
    }

    fn read_enum_variant<T, F>(&mut self, names: &[&str], f: F) -> Result<T, Self::Error> where F: FnMut(&mut Self, usize) -> Result<T, Self::Error> {
        unsupported!(read_enum_variant)
    }

    fn read_enum_variant_arg<T, F>(&mut self, a_idx: usize, f: F) -> Result<T, Self::Error> where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        unsupported!(read_enum_variant_arg)
    }

    fn read_enum_struct_variant<T, F>(&mut self, names: &[&str], f: F) -> Result<T, Self::Error> where F: FnMut(&mut Self, usize) -> Result<T, Self::Error> {
        unsupported!(read_enum_struct_variant)
    }

    fn read_enum_struct_variant_field<T, F>(&mut self, f_name: &str, f_idx: usize, f: F) -> Result<T, Self::Error> where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        unsupported!(read_enum_struct_variant_field)
    }

    fn read_struct<T, F>(&mut self, s_name: &str, len: usize, f: F) -> Result<T, Self::Error> where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        unsupported!(read_struct)
    }

    fn read_struct_field<T, F>(&mut self, f_name: &str, f_idx: usize, f: F) -> Result<T, Self::Error> where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        unsupported!(read_struct_field)
    }

    fn read_tuple<T, F>(&mut self, len: usize, f: F) -> Result<T, Self::Error> where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        unsupported!(read_tuple)
    }

    fn read_tuple_arg<T, F>(&mut self, a_idx: usize, f: F) -> Result<T, Self::Error> where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        unsupported!(read_tuple_arg)
    }

    fn read_tuple_struct<T, F>(&mut self, s_name: &str, len: usize, f: F) -> Result<T, Self::Error> where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        unsupported!(read_tuple_struct)
    }

    fn read_tuple_struct_arg<T, F>(&mut self, a_idx: usize, f: F) -> Result<T, Self::Error> where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        unsupported!(read_tuple_struct_arg)
    }

    fn read_option<T, F>(&mut self, f: F) -> Result<T, Self::Error> where F: FnMut(&mut Self, bool) -> Result<T, Self::Error> {
        unsupported!(read_option)
    }

    fn read_seq<T, F>(&mut self, f: F) -> Result<T, Self::Error> where F: FnOnce(&mut Self, usize) -> Result<T, Self::Error> {
        unsupported!(read_seq)
    }

    fn read_seq_elt<T, F>(&mut self, idx: usize, f: F) -> Result<T, Self::Error> where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        unsupported!(read_seq_elt)
    }

    fn read_map<T, F>(&mut self, f: F) -> Result<T, Self::Error> where F: FnOnce(&mut Self, usize) -> Result<T, Self::Error> {
        unsupported!(read_map)
    }

    fn read_map_elt_key<T, F>(&mut self, idx: usize, f: F) -> Result<T, Self::Error> where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        unsupported!(read_map_elt_key)
    }

    fn read_map_elt_val<T, F>(&mut self, idx: usize, f: F) -> Result<T, Self::Error> where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        unsupported!(read_map_elt_val)
    }

    fn error(&mut self, err: &str) -> Self::Error {
        let error: Box<Error + Send + Sync> = err.to_string().into();
        ::Error::Deserialize(error)
    }
}