// FIXME before release
//#![warn(missing_docs)]

#[macro_use]
extern crate lazy_static;

extern crate hyper;
extern crate mime;
extern crate multipart;
extern crate serde;
extern crate url;

#[macro_export]
pub mod macros;
pub mod net;
pub mod serialize;

use std::error::Error;
use std::marker::PhantomData;
use std::fmt;

pub struct Request<T> {
    _marker: PhantomData<T>,
}

pub enum NeverError {}

impl Error for NeverError {
    fn description(&self) -> &str {
        unreachable!("How the hell did you even call this?!");
    }
}

impl fmt::Debug for NeverError {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unreachable!("How the hell did you even call this?!");
    }
}

impl fmt::Display for NeverError {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unreachable!("How the hell did you even call this?!");
    }
}