extern crate serde_json;

use super::{Serialize, Deserialize};

pub use self::serde_json::Error;

use ::error::map_res;
use ::Result;

use std::io::{Read, Write};

pub struct Serializer;

impl super::Serializer for Serializer {
    fn serialize<T: Serialize, W: Write>(&self, val: &T, write: &mut W) -> Result<()> {
        map_res(self::serde_json::to_writer(write, val))
    }
}

pub struct PrettySerializer;

impl super::Serializer for PrettySerializer {
    fn serialize<T: Serialize, W: Write>(&self, val: &T, write: &mut W) -> Result<()> {
        map_res(self::serde_json::to_writer_pretty(write, val))
    }
}

pub struct Deserializer;

impl super::Deserializer for Deserializer {
    fn deserialize<T: Deserialize, R: Read>(&self, read: &mut R) -> Result<T> {
        map_res(self::serde_json::from_reader(read))
    }
}