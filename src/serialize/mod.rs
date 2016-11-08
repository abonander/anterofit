pub use serde::{Serialize, Deserialize};

use std::error::Error;
use std::io::{Read, Write};
use std::fmt;

mod none;

#[cfg(feature = "json")]
pub mod json;

pub use none::*;

pub trait Serializer: Send + 'static {
    type Error: Into<error::Error>;

    fn serialize<T: Serialize, W: Write>(&self, val: &T, write: &mut W) -> Result<(), Self::Error>;
}

pub trait Deserializer: Send + 'static {
    type Error: Into<error::Error>;

    fn deserialize<T: Deserialize, R: Read>(&self, read: &mut R) -> Result<T, Self::Error>;
}