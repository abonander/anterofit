//! Integration with the `serde_xml` crate providing XML serialization.
//!
//! ##Note
//! As of November 2016, only deserialization is supported by `serde_xml`.

extern crate serde_xml;

use std::io::Read;

use super::Deserialize;

use serialize;
use ::{Error, Result};


/// Deserializer for pulling values from XML responses.
#[derive(Clone, Debug, Default)]
pub struct Deserializer;

impl serialize::Deserializer for Deserializer {
    fn deserialize<T: Deserialize, R: Read>(&self, read: &mut R) -> Result<T> {
        Error::map_deserialize(self::serde_xml::de::from_iter(read.bytes()))
    }
}