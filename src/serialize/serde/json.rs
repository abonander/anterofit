//! Integration with the `serde_json` crate providing JSON serialization.

extern crate serde_json;

use mime::{self, Mime};

use std::io::{Read, Write};

use super::{Serialize, Deserialize};

use ::error::map_res;
use ::Result;

pub use self::serde_json::Error;

/// Serializer for JSON request bodies with compact output.
pub struct Serializer;

impl super::Serializer for Serializer {
    fn serialize<T: Serialize, W: Write>(&self, val: &T, write: &mut W) -> Result<()> {
        map_res(self::serde_json::to_writer(write, val))
    }

    /// Returns `application/json`.
    fn content_type(&self) -> Option<Mime> {
        Some(mime::json())
    }
}

/// Serializer for JSON request bodies which pretty-prints its output.
pub struct PrettySerializer;

impl super::Serializer for PrettySerializer {
    fn serialize<T: Serialize, W: Write>(&self, val: &T, write: &mut W) -> Result<()> {
        map_res(self::serde_json::to_writer_pretty(write, val))
    }

    fn content_type(&self) -> Option<Mime> {
        Some(mime::json())
    }
}

/// Deserializer for pulling values from JSON response bodies.
pub struct Deserializer;

impl super::Deserializer for Deserializer {
    fn deserialize<T: Deserialize, R: Read>(&self, read: &mut R) -> Result<T> {
        map_res(self::serde_json::from_reader(read))
    }
}