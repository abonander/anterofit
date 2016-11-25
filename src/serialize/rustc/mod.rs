//! Serialization with [`rustc-serialize`](https://github.com/rust-lang-nursery/rustc-serialize)
//!
pub use rustc_serialize::{
    Decodable as Deserialize,
    Encodable as Serialize,
    Decoder,
    Encoder
};

pub mod json;