//! Serialization with [Serde](https://serde.rs)

extern crate serde;

pub use self::serde::{Serialize, Deserialize};

#[cfg(feature = "serde_json")]
pub mod json;

#[cfg(feature = "serde_xml")]
pub mod xml;