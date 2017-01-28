//! Serialization with [Serde](https://serde.rs)

extern crate serde;

pub use self::serde::{Serialize, Deserialize};

use self::serde::{Serializer, Deserializer};
use self::serde::de::{Error, Visitor};

use super::FromStrDeserializerImpl;

use std::error::Error as StdError;
use std::fmt::{Display, Write};
use std::io::Read;

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

impl Error for ::Error {
    fn custom<T: Display>(msg: T) -> Self {
        let error: Box<Error + Send + Sync> = msg.to_string().into();
        ::Error::Deserialize(error)
    }
}

impl<R: Read> Deserializer for FromStrDeserializerImpl<R> {
    type Error = ::Error;

    fn deserialize<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        panic!("FromStrDeserializer cannot guess types. Visitor expecting")
    }

    fn deserialize_bool<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        let val = try!(self.read_val());
        visitor.visit_bool(val)
    }

    fn deserialize_u8<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        visitor.visit_u8(try!(self.read_val()))
    }

    fn deserialize_u16<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        visitor.visit_u16(try!(self.read_val()))
    }

    fn deserialize_u32<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        visitor.visit_u32(try!(self.read_val()))
    }

    fn deserialize_u64<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        visitor.visit_u64(try!(self.read_val()))
    }

    fn deserialize_i8<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        visitor.visit_i8(try!(self.read_val()))
    }

    fn deserialize_i16<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        visitor.visit_i16(try!(self.read_val()))
    }

    fn deserialize_i32<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        visitor.visit_i32(try!(self.read_val()))
    }

    fn deserialize_i64<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        visitor.visit_i64(try!(self.read_val()))
    }

    fn deserialize_f32<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        visitor.visit_f32(try!(self.read_val()))
    }

    fn deserialize_f64<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        visitor.visit_f64(try!(self.read_val()))
    }

    fn deserialize_char<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        visitor.visit_char(try!(self.read_char()))
    }

    fn deserialize_str<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        visitor.visit_str(&try!(self.read_string()))
    }

    fn deserialize_string<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        visitor.visit_string(try!(self.read_string()))
    }

    fn deserialize_unit<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        unimplemented!()
    }

    fn deserialize_option<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        unimplemented!()
    }

    fn deserialize_seq<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        unimplemented!()
    }

    fn deserialize_seq_fixed_size<V>(&mut self, len: usize, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        unimplemented!()
    }

    fn deserialize_bytes<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        unimplemented!()
    }

    fn deserialize_map<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        unimplemented!()
    }

    fn deserialize_unit_struct<V>(&mut self, name: &'static str, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        unimplemented!()
    }

    fn deserialize_newtype_struct<V>(&mut self, name: &'static str, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        unimplemented!()
    }

    fn deserialize_tuple_struct<V>(&mut self, name: &'static str, len: usize, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        unimplemented!()
    }

    fn deserialize_struct<V>(&mut self, name: &'static str, fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        unimplemented!()
    }

    fn deserialize_struct_field<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        unimplemented!()
    }

    fn deserialize_tuple<V>(&mut self, len: usize, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        unimplemented!()
    }

    fn deserialize_enum<V>(&mut self, name: &'static str, variants: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        unimplemented!()
    }

    fn deserialize_ignored_any<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor {
        unimplemented!()
    }
}