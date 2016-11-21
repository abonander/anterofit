//! Wrap REST calls with Rust traits.
//! 
//! ##Example
//! ```rust
//! // N.B.: this requires nightly to build because of the `proc_macro` feature. However,
//! // this is only necessary for the sake of brevity: on the stable and beta channels,
//! // you can use `serde_codegen` and a build script to generate a `Deserialize` impl
//! // as described here: https://serde.rs/codegen-stable.html.
//! #![feature(proc_macro)]
//! 
//! #[macro_use]
//! extern crate anterofit;
//! 
//! // If you get a "not found" error here, enable the `nightly` feature (nightly channel required)
//! #[macro_use]
//! extern crate serde_derive;
//! 
//! use anterofit::*;
//! 
//! #[derive(Debug, Deserialize)]
//! pub struct Post {
//!     pub userid: Option<u64>,
//!     pub id: u64,
//!     pub title: String,
//!     pub body: String
//! }
//! 
//! service! {
//!     pub trait TestService {
//!         get! {
//!             fn get_post(&self, id: u64) -> Post {
//!                 url = "/posts/{}", id
//!             }
//!         }
//! 
//!         get! {
//!             fn get_posts(&self) -> Vec<Post> {
//!                 url = "/posts"
//!             }
//!         }
//!     }
//! }
//! 
//! fn main() {
//!     let url = Url::parse("https://jsonplaceholder.typicode.com").unwrap();
//! 
//!     let adapter = Adapter::builder()
//!         .base_url(url)
//!         // If you get a "not found" error from the compiler here, enable the `json` feature.
//!         .deserialize(anterofit::serialize::json::Deserializer)
//!         .build();
//! 
//!     fetch_posts(&adapter);
//! }
//! 
//! fn fetch_posts<T: TestService>(test_service: &T) {
//!     let posts = test_service.get_posts()
//!         .exec_here()
//!         .unwrap();
//! 
//!     for post in posts.into_iter().take(3) {
//!         println!("{:?}", post);
//!     }
//! }
//! ```
#![warn(missing_docs)]
#![cfg_attr(feature = "nightly", feature(insert_str))]
#![recursion_limit="100"]

#[macro_use]
extern crate quick_error;

extern crate futures;

#[macro_use]
extern crate mime as mime_;

extern crate multipart;
extern crate serde;

extern crate url;

pub extern crate hyper;

mod mime;

pub mod macros;

pub mod net;
pub mod serialize;

pub mod executor;

pub mod error;

pub use error::Error;
pub use error::Never as NeverError;

pub use hyper::Url;

pub use net::Adapter;

pub use net::RawBody;

/// The result type for this crate; used frequently in public APIs.
///
/// Recommended to be used as `anterofit::Result` to avoid confusing
/// shadowing of `std::result::Result`.
pub type Result<T> = ::std::result::Result<T, Error>;





