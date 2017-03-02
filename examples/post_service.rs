// This example assumes the `rustc-serialize` feature.
//
// If you are using the `serde` feature, use `#[derive(Deserialize)]`
// and `serialize::serde::json::Deserializer` instead at the appropriate places.

#[macro_use] extern crate anterofit;
extern crate rustc_serialize;

// The minimum imports needed to get this example working.
//
// You can glob-import if you like, but know that it will shadow `Result`
// which may cause some confusing type-mismatch errors.
use anterofit::{Adapter, Url};

#[derive(Debug, RustcDecodable)]
struct Post {
    pub userid: Option<u64>,
    pub id: u64,
    pub title: String,
    pub body: String
}

/// Used to create a new Post.
#[derive(Debug, RustcEncodable)]
struct NewPost<'a> {
    pub userid: u64,
    pub title: &'a str,
    pub body: &'a str,
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

        /// Get all posts by a specific user
        fn posts_by_user(&self, userid: u64) -> Vec<Post> {
            GET("/user/{}/posts", userid)
        }

        // TODO: demonstrate `query!{}`

        /// Create a new Post under the given user ID with the given title and body.
        fn new_post(&self, userid: u64, title: &str, body: &str) -> Post {
            POST("/posts/");
            // We use the `EAGER:` keyword so we can use borrowed values in the body.
            // This serializes the body value immediately instead of waiting to serialize
            // it on the executor.
            body!(EAGER: NewPost {
                userid: userid,
                title: title,
                body: body
            })
        }
    }
}

service_delegate!(PostService);

fn main() {
    // Navigate to this URL in your browser for details. Very useful test API.
    let url = Url::parse("https://jsonplaceholder.typicode.com").unwrap();

    let adapter = Adapter::builder()
        .base_url(url)
        // When your REST API uses JSON in both requests and responses
        .serialize_json()
        .build();

    let service = adapter.arc_service::<PostService>();

    create_post(&adapter);
    fetch_posts(&adapter);
    user_posts(&*service);
}

/// Create a new Post.
fn create_post<P: PostService>(post_service: &P) {
    let post = post_service.new_post(42, "Hello", "World!")
        // If you don't want to block, the return value of exec() can be used as a Future
        // to poll for the result. However, it does shadow a couple methods of Future
        // so that you don't have to import the trait to use them.
        // See the docs of Call for more info.
        .exec().block()
        .unwrap();

    println!("Created post: {:?}", post);
}

/// Fetch the top 3 posts in the database.
// Service traits can be object-safe
fn fetch_posts(post_service: &PostService) {
    let posts = post_service.get_posts()
        // Shorthand for .exec().wait(), but executes the request on the current thread.
        .exec_here()
        .unwrap();

    for post in posts.into_iter().take(3) {
        println!("Fetched post: {:?}", post);
    }
}

fn user_posts(post_service: &PostService) {
    post_service.posts_by_user(1)
        // This will be executed asynchronously when the request is completed
        .on_complete(|posts| for post in posts { println!("User post: {:?}", post); })
        .exec().ignore();
}