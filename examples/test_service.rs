#![feature(proc_macro)]
#![cfg(feature = "json")]

#[macro_use]
extern crate anterofit;

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