pub use serde::{Serialize, Deserialize};

use mime::Mime;

use std::io::{Read, Write};

use ::Result;

mod none;

pub use self::none::*;

pub trait Serializer: Send + Sync + 'static {
    fn serialize<T: Serialize, W: Write>(&self, val: &T, write: &mut W) -> Result<()>;

    fn content_type(&self) -> Option<Mime>;
}

pub trait Deserializer: Send + Sync + 'static {
    fn deserialize<T: Deserialize, R: Read>(&self, read: &mut R) -> Result<T>;
}

macro_rules! modules {
    ($($name:ident = $strname:expr),*) => (
        $(
            #[cfg(feature = $strname)]
            pub mod $name;

            #[cfg(not(feature = $strname))]
            pub mod $name {
                /// Empty error type to fill the associated variant of `error::Error`.
                quick_error! {
                    #[derive(Debug)]
                    pub enum Error {}
                }
            }
        )*
    )
}

modules! {
    json = "json", xml = "xml"
}
