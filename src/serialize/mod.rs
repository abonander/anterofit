pub use serde::{Serialize, Deserialize};

use mime::Mime;

use std::io::{Read, Write};

use ::Result;

mod none;

#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "xml")]
pub mod xml;

pub use self::none::*;

pub trait Serializer: Send + Sync + 'static {
    fn serialize<T: Serialize, W: Write>(&self, val: &T, write: &mut W) -> Result<()>;

    fn content_type(&self) -> Option<Mime>;
}

pub trait Deserializer: Send + Sync + 'static {
    fn deserialize<T: Deserialize, R: Read>(&self, read: &mut R) -> Result<T>;
}