pub use serde::{Serialize, Deserialize};

use std::io::{Read, Write};

use ::Result;

mod none;

#[cfg(feature = "json")]
pub mod json;

pub use self::none::*;

pub trait Serializer: Send + Sync + 'static {
    fn serialize<T: Serialize, W: Write>(&self, val: &T, write: &mut W) -> Result<()>;
}

pub trait Deserializer: Send + Sync + 'static {
    fn deserialize<T: Deserialize, R: Read>(&self, read: &mut R) -> Result<T>;
}