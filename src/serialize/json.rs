extern crate serde_json;

use super::{Serialize, Deserialize};

pub use self::serde_json::Error;

use std::io::{Read, Write};

pub struct Serializer;

impl super::Serializer for Serializer {
    type Error = Error;

    fn serialize<T: Serialize, W: Write>(&self, val: &T, write: &mut W) -> Result<(), Error> {
        self::serde_json::to_writer(write, val)
    }
}

pub struct PrettySerializer;

impl super::Serializer for PrettySerializer {
    type Error = Error;

    fn serialize<T: Serialize, W: Write>(&self, val: &T, write: &mut W) -> Result<(), Error> {
        self::serde_json::to_writer_pretty(write, val)
    }
}

pub struct Deserializer;

impl super::Deserializer for Deserializer {
    type Error = Error;

    fn deserialize<T: Deserialize, R: Read>(&self, read: &mut R) -> Result<T, Error> {
        self::serde_json::from_reader(read)
    }
}