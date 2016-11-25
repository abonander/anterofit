// This example assumes the `rustc-serialize` feature.
//
// If you are using the `serde` feature, use `#[derive(Deserialize)]`
// and `serialize::serde::json::Deserializer` instead at the appropriate places.

#[macro_use]
extern crate anterofit;

extern crate rustc_serialize;

use anterofit::*;

#[derive(Debug, RustcDecodable)]
pub struct Post {
    pub userid: Option<u64>,
    pub id: u64,
    pub title: String,
    pub body: String
}

service! {
    pub trait PostService {
        get! {
            fn get_post(&self, id: u64) -> Post {
                url("/posts/{}", id)
            }
        }

        get! {
            fn get_posts(&self) -> Vec<Post> {
                url("/posts")
            }
        }
    }
}

fn main() {
    let url = Url::parse("https://jsonplaceholder.typicode.com").unwrap();

    let adapter = Adapter::builder()
        .base_url(url)
        .deserialize(serialize::rustc::json::Deserializer)
        .build();

    fetch_posts(&adapter);
}

fn fetch_posts<T: PostService>(post_service: &T) {
    let posts = post_service.get_posts()
        .exec_here()
        .unwrap();

    for post in posts.into_iter().take(3) {
        println!("{:?}", post);
    }
}