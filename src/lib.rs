//! Wrap REST calls with Rust traits.
//! 
//! ##Example
//! ```rust
//! // This example assumes the `rustc-serialize` feature.
//! //
//! // If you are using the `serde` feature, use `#[derive(Deserialize)]`
//! // and `serialize::serde::json::Deserializer` instead at the appropriate places.
//!
//! #[macro_use]
//! extern crate anterofit;
//!
//! extern crate rustc_serialize;
//!
//! use anterofit::*;
//!
//! #[derive(Debug, RustcDecodable)]
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
//!     let url = Url::parse("https://! jsonplaceholder.typicode.com").unwrap();
//!
//!     let adapter = Adapter::builder()
//!         .base_url(url)
//!         .deserialize(serialize::rustc::json::Deserializer)
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

extern crate url;

pub extern crate hyper;

mod mime;

mod macros;

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





