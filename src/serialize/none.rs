//! No-op serializers which return errors when invoked.

use std::io::{Read, Write};

use super::{Serializer, Deserializer, Serialize, Deserialize};

use mime::Mime;

use ::Result;

/// A no-op serializer which returns an error when attempting to use it.
pub struct NoSerializer;

impl Serializer for NoSerializer {
    fn serialize<T: Serialize, W: Write>(&self, _: &T, _: &mut W) -> Result<()> {
        Err(NoSerializeError::Serialize.into())
    }

    fn content_type(&self) -> Option<Mime> {
        None
    }
}

/// A no-op deserializer which returns an error when attempting to use it.
pub struct NoDeserializer;

impl Deserializer for NoDeserializer {
    fn deserialize<T: Deserialize, R: Read>(&self, _: &mut R) -> Result<T> {
        Err(NoSerializeError::Deserialize.into())
    }
}

quick_error! {
    /// Error returned by `NoSerializer` and `NoDeserializer`
    #[derive(Debug)]
    pub enum NoSerializeError {
        /// "A request method requested serialization, but no serializer was provided"
        Serialize {
            description("A request method requested serialization, but no serializer was provided")
        }
        /// "A request method requested deserialization, but no deserializer was provided"
        Deserialize {
            description("A request method requested deserialization, but no deserializer was provided")
        }
    }
}