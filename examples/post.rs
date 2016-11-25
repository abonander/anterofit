// This example assumes the `rustc-serialize` feature.
//
// If you are using the `serde` feature, use `#[derive(Deserialize)]`
// and `serialize::serde::json::Deserializer` instead at the appropriate places.

#[macro_use] extern crate anterofit;

extern crate rustc_serialize;

use anterofit::*;

#[derive(Debug, RustcDecodable)]
pub struct Post {
    pub userid: Option<u64>,
    pub id: u64,
    pub title: String,
    pub body: String
}

#[derive(Debug, RustcEncodable)]
pub struct NewPost<'a> {
    pub userid: u64,
    pub title: &'a str,
    pub body: &'a str,
}

service! {
    pub trait PostService {
        #[POST("/posts/")]
        fn new_post(&self, userid: u64, title: &str, body: &str) -> Post {
            // We use body_eager! so we can use borrowed values in the body.
            // It requires access to the adapter, so we pass `self` as the first argument.
            body_eager!(NewPost {
                userid: userid,
                title: title,
                body: body
            })
        }
    }
}

fn main() {
    let url = Url::parse("https://jsonplaceholder.typicode.com").unwrap();

    let adapter = Adapter::builder()
        .base_url(url)
        .deserialize(serialize::rustc::json::Deserializer)
        .build();

    create_post(&adapter);
}

fn create_post<T: PostService>(post_service: &T) {
    let post = post_service.new_post(42, "Hello", "World!")
        .exec_here()
        .unwrap();

    println!("{:?}", post);
}