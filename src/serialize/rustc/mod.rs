//! Serialization with [`rustc-serialize`](https://github.com/rust-lang-nursery/rustc-serialize).

pub use rustc_serialize::{
    Decodable,
    Encodable,
    Decoder,
    Encoder
};

pub mod json;