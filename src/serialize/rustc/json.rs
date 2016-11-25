//! JSON serialization with `rustc-serialize`

use ::serialize::{Serialize, Deserialize};
use ::mime::Mime;
use ::{Error, Result};

use rustc_serialize::json::{Encoder, Decoder, Json};

use std::io::{self, Read, Write};
use std::fmt;

/// Compact-format JSON serializer.
pub struct Serializer;

impl ::serialize::Serializer for Serializer {
    fn serialize<T: Serialize, W: Write>(&self, val: &T, write: &mut W) -> Result<()> {
        let mut adapter = FmtAdapter::new(write);

        try!(Error::map_serialize(val.encode(&mut Encoder::new(&mut adapter))));

        adapter.into_result()
    }

    /// Return the MIME type of the serialized content, if applicable.
    ///
    /// Used to set the `Content-Type` header of the request this serializer
    /// is being used for.
    fn content_type(&self) -> Option<Mime> {
        Some(::mime::json())
    }
}

/// Pretty-printing JSON serializer with configurable indent.
pub struct PrettySerializer(Option<u32>);

impl PrettySerializer {
    /// Create a new pretty-printer with the default indent style.
    pub fn new() -> Self {
        PrettySerializer(None)
    }

    /// Create a new pretty-printer which indents by the given number of spaces each level.
    pub fn with_indent(num_spaces: u32) -> Self {
        PrettySerializer(Some(num_spaces))
    }
}

impl ::serialize::Serializer for PrettySerializer {
    fn serialize<T: Serialize, W: Write>(&self, val: &T, write: &mut W) -> Result<()> {
        let mut adapter = FmtAdapter::new(write);

        {
            let mut encoder = Encoder::new_pretty(&mut adapter);

            if let Some(num_spaces) = self.0 {
                encoder.set_indent(num_spaces)
                    .expect("Make sure the line above this is `new_pretty()`");
            }

            try!(Error::map_serialize(val.encode(&mut encoder)));
        }

        adapter.into_result()
    }

    /// Return the MIME type of the serialized content, if applicable.
    ///
    /// Used to set the `Content-Type` header of the request this serializer
    /// is being used for.
    fn content_type(&self) -> Option<Mime> {
        Some(::mime::json())
    }
}

// Adapted from https://github.com/rust-lang/rust/blob/master/src/libstd/io/mod.rs#L997
// Why they don't just make it a public type, I will never know.
struct FmtAdapter<'a, W: 'a> {
    write: &'a mut W,
    res: io::Result<()>,
}

impl<'a, W: 'a> FmtAdapter<'a, W> {
    fn new(write: &'a mut W) -> Self {
        FmtAdapter {
            write: write,
            res: Ok(())
        }
    }

    fn into_result(self) -> Result<()> {
        Error::map_serialize(self.res)
    }
}

impl<'a, W: Write + 'a> fmt::Write for FmtAdapter<'a, W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self.write.write_all(s.as_bytes()) {
            Ok(()) => Ok(()),
            e => {
                self.res = e;
                Err(fmt::Error)
            }
        }
    }
}

/// JSON deserializer.
pub struct Deserializer;

impl ::serialize::Deserializer for Deserializer {
    fn deserialize<T: Deserialize, R: Read>(&self, read: &mut R) -> Result<T> {
        let json = try!(Error::map_deserialize(Json::from_reader(read)));
        let mut decoder = Decoder::new(json);
        Error::map_deserialize(T::decode(&mut decoder))
    }
}