# anterofit

##### [retrofit](http://www.merriam-webster.com/dictionary/retrofit) | *verb*  \ˈre-trō-ˌfit, ˌre-trō-ˈfit\

*3: to adapt to a new purpose or need*

*?: a [Java library](https://square.github.io/retrofit) which generates REST API wrappers at runtime using class metadata* 

##### anterofit | *verb (neologism)* \ˈan-tə-(ˌ)rō-ˌfit\

*?: to create an original implement to suit a particular purpose or need*

*?: a Rust crate which generates type-safe REST API wrappers at compile time using macros*

---

### `test_service.rs`

This example links with the test JSON-based REST API at https://jsonplaceholder.typicode.com:

```rust 
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
