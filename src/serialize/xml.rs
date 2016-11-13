//! Integration with the `serde_xml` crate providing XML serialization.
//!
//! ##Note
//! As of November 2016, only deserialization is supported by `serde_xml`.

extern crate serde_xml;

use std::io::Read;

use super::Deserialize;

use ::error::map_res;
use ::Result;

pub use self::serde_xml::Error;

pub struct Deserializer;

impl super::Deserializer for Deserializer {
    fn deserialize<T: Deserialize, R: Read>(&self, read: &mut R) -> Result<T> {
        map_res(self::serde_xml::from_reader(read))
    }
}