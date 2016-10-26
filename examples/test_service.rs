#![feature(proc_macro)]
#![cfg(feature = "json")]

#[macro_use]
extern crate anterofit;

#[macro_use]
extern crate serde_derive;

use anterofit::net::Adapter;

#[derive(Deserialize)]
pub struct Post {
    pub userid: u64,
    pub id: u64,
    pub title: String,
    pub body: String
}

service! {
    pub trait TestService {
        get! {
            fn get_post(id: u64) -> Post {
                url = "/posts/{}", id
            }
        }

        get! {
            fn get_posts() -> Vec<Post> {
                url = "/posts"
            }
        }
    }
}

fn main() {
    Adapter::builder("https://jsonplaceholder.typicode.com/")
        .deserialize(anterofit::serialize::json::Serializer)
        .build()
}