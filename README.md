# anterofit [![Build Status](https://travis-ci.org/abonander/anterofit.svg?branch=master)](https://travis-ci.org/abonander/anterofit) [![On Crates.io](https://img.shields.io/crates/v/anterofit.svg)](https://crates.io/crates/anterofit)

Anterofit is a collection of Rust macros coupled to a lightweight, self-contained HTTP framework that
allows you to create strongly-typed Rust wrappers around REST APIs with ease.

```rust
// See examples/post_service.rs for more details
#[macro_use] extern crate anterofit;
extern crate rustc_serialize;

use anterofit::{Adapter, Url};

#[derive(Debug, RustcDecodable)]
struct Post {
    pub userid: Option<u64>,
    pub id: u64,
    pub title: String,
    pub body: String
}

service! {
    trait PostService {
        /// Get a Post by id.
        fn get_post(&self, id: u64) -> Post {
            GET("/posts/{}", id)
        }

        /// Get all posts.
        fn get_posts(&self) -> Vec<Post> {
            GET("/posts")
        }

        /// Create a new Post under the given user ID with the given title and body.
        fn new_post(&self, userid: u64, title: &str, body: &str) -> Post {
            POST("/posts/");
            // We use the `EAGER:` keyword so we can use borrowed values in the body.
            // This serializes the body value immediately instead of waiting to serialize
            // it on the executor.
            body_map!(EAGER:
                "userid" => userid,
                "title": title,
                "body": body
            )
        }
    }
}

fn main() {
    // Navigate to this URL in your browser for details. Very useful test API.
    let url = Url::parse("https://jsonplaceholder.typicode.com").unwrap();

    let adapter = Adapter::builder()
        .base_url(url)
        // When your REST API uses JSON in both requests and responses
        .serialize_json()
        .build();

    create_post(&adapter);
    fetch_posts(&adapter);
}

/// Create a new Post.
// All service traits are implemented for `Adapter` by default; using generics like this promotes good namespacing.
fn create_post<P: PostService>(post_service: &P) {
    let post = post_service.new_post(42, "Hello", "World!")
        // Execute the request in the background and wait for it to complete
        .exec().block()
        .unwrap();

    println!("{:?}", post);
}

/// Fetch the top 3 posts in the database.
// Service traits are object-safe by default
fn fetch_posts(post_service: &PostService) {
    let posts = post_service.get_posts()
        // Shorthand for .exec().block(), but executes the request on the current thread.
        .exec_here()
        .unwrap();

    for post in posts.into_iter().take(3) {
        println!("{:?}", post);
    }
}
```

Inspired by [Square's Retrofit](https://sqaure.github.io/retrofit), as referenced in the name, Anterofit is even
more strongly typed as everything that is feasible to check at compile-time, is. Runtime errors are,
with few exceptions, reserved for error conditions that can only be discovered at runtime.

Usage
-----

Get started with our [User Guide](GUIDE.md)

Or an in-depth look with our [Documentation](https://docs.rs/anterofit)

###Setup

#### [Serde](https://crates.io/crates/serde) and JSON serialization:

Enabled by default with the `serde-all` feature.

`Cargo.toml`:
```toml
[dependencies]
anterofit = "0.1"
serde = "0.9"
serde_json = "0.9"
serde_derive = "0.9"
```

Crate Root:
```rust
#[macro_use] extern crate anterofit;
extern crate serde;
#[macro_use] extern crate serde_derive;
```

####[`rustc-serialize`](https://crates.io/crates/rustc-serialize):

`Cargo.toml`:
```toml
[dependencies]
rustc-serialize = "0.3"

[dependencies.anterofit]
version = "0.1"
default-features = false
features = ["rustc-serialize"]
```

Crate Root:
```rust
#[macro_use] extern crate anterofit;
extern crate rustc_serialize;
```

License
-------

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.