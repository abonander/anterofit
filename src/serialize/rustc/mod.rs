//! Serialization with [`rustc-serialize`](https://github.com/rust-lang-nursery/rustc-serialize)

extern crate rustc_serialize;

pub use self::rustc_serialize::{
    Decodable as Deserialize,
    Encodable as Serialize,
    Decoder,
    Encoder
};

pub mod json;