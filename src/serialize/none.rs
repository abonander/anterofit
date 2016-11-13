use std::error::Error;
use std::fmt;
use std::io::{Read, Write};

use super::{Serializer, Deserializer, Serialize, Deserialize};

use mime::Mime;

use ::Result;

/// A no-op serializer which returns an error when attempting to use it.
pub struct NoSerializer;

/// Returned by `<NoSerializer as Serializer>::serialize()`.
///
/// Usually means you tried to serialize a type as a request body without supplying
/// a serializer when building the adapter you used.
#[derive(Debug)]
pub struct NoSerializerError(());

impl Serializer for NoSerializer {
    fn serialize<T: Serialize, W: Write>(&self, _: &T, _: &mut W) -> Result<()> {
        Err(NoSerializerError(()).into())
    }

    fn content_type(&self) -> Option<Mime> {
        None
    }
}

impl Error for NoSerializerError {
    fn description(&self) -> &str {
        "No serializer was provided in the RequestAdapter."
    }
}

impl fmt::Display for NoSerializerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

/// A no-op deserializer which returns an error when attempting to use it.
pub struct NoDeserializer;

/// Returned by `<NoDeserializer as Deerializer>::deserialize()`.
///
/// Usually means you tried to deserialize a type from a response body without supplying
/// a deserializer when building the adapter you used.
#[derive(Debug)]
pub struct NoDeserializerError(());

impl Error for NoDeserializerError {
    fn description(&self) -> &str {
        "No deserializer was provided in the RequestAdapter."
    }
}

impl fmt::Display for NoDeserializerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl Deserializer for NoDeserializer {
    fn deserialize<T: Deserialize, R: Read>(&self, _: &mut R) -> Result<T> {
        Err(NoDeserializerError(()).into())
    }
}