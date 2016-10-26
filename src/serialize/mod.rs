pub use serde::{Serialize, Deserialize};

use std::error::Error;
use std::io::{Read, Write};
use std::fmt;

#[cfg(feature = "json")]
pub mod json;

pub trait Serializer: Send + 'static {
    type Error: Error + Send + 'static;

    fn serialize<T: Serialize, W: Write>(&self, val: &T, write: &mut W) -> Result<(), Self::Error>;
}

pub trait Deserializer: Send + 'static {
    type Error: Error + Send + 'static;

    fn deserialize<T: Deserialize, R: Read>(&self, read: &mut R) -> Result<T, Self::Error>;
}

pub struct NoSerializer;

#[derive(Debug)]
pub struct NoSerializerError(());

impl Serializer for NoSerializer {
    type Error = NoSerializerError;

    fn serialize<T: Serialize, W: Write>(&self, _: &T, _: &mut W) -> Result<(), Self::Error> {
        Err(NoSerializerError(()))
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

pub struct NoDeserializer;

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
    type Error = NoDeserializerError;

    fn deserialize<T: Deserialize, R: Read>(&self, read: &mut R) -> Result<T, Self::Error> {
        Err(NoDeserializerError(()))
    }
}