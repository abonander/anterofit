# anterofit

#### [retrofit](https://square.github.io/retrofit) | *verb*  \ˈre-trō-ˌfit, ˌre-trō-ˈfit\

* *to adapt an existing implement to suit a new purpose or need*

* *a Java library which generates REST API wrappers at runtime using class metadata* 

#### anterofit | *verb (neologism)* \ˈan-tə-(ˌ)rō-ˌfit\

* *to create an original implement to suit a particular purpose or need*

* *a Rust crate which generates type-safe REST API wrappers at compile time using macros*

Anterofit is a collection of Rust macros coupled to a lightweight, self-contained HTTP framework that
allows you to create strongly-typed wrappers around Rust APIs with ease.

Inspired by [Square's Retrofit](https://sqaure.github.io/retrofit), as referenced in the name, Anterofit is even
more strongly typed as everything that is feasible to check at compile-time, is. Runtime errors are,
with few exceptions, reserved for error conditions that can only be discovered at runtime.

---

### `test_service.rs`

This example links with the test JSON-based REST API at https://jsonplaceholder.typicode.com:

**N.B.**: this requires nightly to build because of the `proc_macro` feature. However,
this is only necessary for the sake of brevity: on the stable and beta channels, you can use `serde_codegen`
and a build script to generate this implementation as described [here](https://serde.rs/codegen-stable.html).

```rust 
#![feature(proc_macro)]

#[macro_use]
extern crate serde_derive;

use anterofit::*;

#[derive(Debug, Deserialize)]
pub struct Post {
    pub userid: Option<u64>,
    pub id: u64,
    pub title: String,
    pub body: String
}

service! {
    pub trait TestService {
        get! {
            fn get_post(&self, id: u64) -> Post {
                url = "/posts/{}", id
            }
        }

        get! {
            fn get_posts(&self) -> Vec<Post> {
                url = "/posts"
            }
        }
    }
}

fn main() {
    let url = Url::parse("https://jsonplaceholder.typicode.com").unwrap();

    let adapter = Adapter::builder(url)
        .deserialize(anterofit::serialize::json::Deserializer)
        .build();

    fetch_posts(&adapter);
}

fn fetch_posts<T: TestService>(test_service: &T) {
    let posts = test_service.get_posts()
        .here()
        .unwrap();

    for post in posts.into_iter().take(3) {
        println!("{:?}", post);
    }
}
```
