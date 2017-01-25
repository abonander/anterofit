Anterofit Release Announcement
================

The common practice when releasing any relatively novel crate is to write a blog post
about the subject. However, since I don't have a blog set up--and, to be brutally honest,
I find the concept of having a personal blog rather pretentious--I figured I'd just write the release
announcement in the repo itself.

Anterofit has been a long time coming. I've been conceptualizing it ever since I started tinkering with Rust,
something like over three years ago. For the longest time, I thought it wasn't possible without compiler
plugins, which dampened my enthusiasm somewhat as they are perpetually unstable and break occasionally without much 
warning. As it turns out, I was partially right, as the macro-based implementation has one big, ugly limitation, 
but the future is looking bright.

Inspiration
-----------

I basically straight-up copied the concept behind [Retrofit][retrofit], a
Java library I have been using for many years. It has great utility in Android
development, where there is often a want for an app to talk to a server backend
to store user data and things of that nature.

The basic idea is that you write a Java interface describing a REST API (which can, and should,
be split across multiple interfaces),
using annotations to add the necessary information to construct a request, and then
Retrofit constructs an instance of that interface such that calling methods on it
issues requests and returns their responses. Serialization is supported in both directions,
so you can set any serializable object as the request body and have the response be deserialized
to some POD object.

For a useful example, the interface and usage to fetch a post from the [JSONPlaceholder API][jsonplaceholder], 
which has been immensely useful in developing and testing Anterofit, would look something like this:

```java
public class Post {
    public long userId;
    public long id;
    public String title;
    public String body;
}

// Calling it a "service" comes from Retrofit's only example on interface declaration
// I've used the same convention ever since
public interface PostService {
    @POST("posts")
    Call<Post> createPost(@Field("userid") long userId, @Field("title") String title, 
                          @Field("body") String Body);
    
    @GET("posts/{id}")
    Call<Post> getPost(@Path("id") long id);
}
```

And then to use it, you construct an instance of the `Retrofit` class and have it create
an object from the interface class and call the declared methods on it:

```java
Retrofit retrofit = new Retrofit.Builder()
    .baseUrl("https://jsonplaceholder.typicode.com/")
    // You actually need to supply a converter to parse JSON responses into objects
    // That would be done before calling `.build()`
    .build();

PostService service = retrofit.create(PostService.class);

Call<Post> newPost = service.createPost(42, "Hello, world!", "Lorem ipsum dolor sit amet");

// The JSONPlaceholder API doesn't save anything for obvious reasons;
// this will return a Post instance with some filler text instead.
Call<Post> post = service.getPost(0);
```

One shortcoming of Retrofit's documentation is that it doesn't show how to execute
the `Call` instance to access the response. In the version of Retrofit that I'm used to
using (v1.x), you either add an extra parameter for a callback instance and declare the return type `void`,
or use the return type directly (which makes the request synchronous). This makes things a little more clear,
but if you want an asynchronous and a synchronous call to the same endpoint, you need to declare two different
methods, which is kind of ugly. 

Anterofit functions similarly to how Retrofit does now, but hopefully the documentation is sufficiently
detailed to avoid confusion.
 
[retrofit]: http://square.github.io/retrofit/
[jsonplaceholder]: http://jsonplaceholder.typicode.com
 
Introducing Anterofit
---------------------

Anterofit is a collection of macros that makes it easy to wrap REST APIs using Rust traits. Superficially,
 it functions similarly to Retrofit, but instead of constructing trait instances at runtime (which would
 be hacky and full of overhead), Anterofit simply rewrites the trait declarations and injects all the
 necessary details for making a request and parsing the response. This makes the implementation
 much less magical, more approachable and more maintainable. Also, much easier to document.
 
 Additionally, Anterofit is futures-aware (`Call<T>` implements `Future<Item = T>`), so that you can poll 
 for the status of requests from an event loop. However, for standalone use, it also provides inherent methods
 equivalent to  `Future::poll()` and `Future::wait()` without requiring use of the `futures` crate. I have great
 plans for futures integration; see the [Looking to the Future / Tokio](#tokio) section for details.

The name didn't really sound "cool" when I first came up with it, but it's grown on me. Retrofit is an
appropriate name for a framework which does all the setup at runtime, so I figured that since
Anterofit does almost all the setup at compile time instead (in true Rust fashion), 
I should try to find (or construct) the antonym of "retrofit". I would like to say I coined "anterofit"
 out of my own brilliance, by realizing the opposite of "retro-" could be considered to be "antero-", 
 but I didn't. I stole the word from [this StackExchange answer](http://english.stackexchange.com/a/150352) instead. 
 Technically, doesn't it exist now that there's a semi-legitimate usage for it?
 

How It Works
------------

I have written a [User's Guide](GUIDE.md) going into much more detail, but here's the basics:

* Use the `service!{}` macro to write a trait interface describing a REST API endpoint (for example, the equivalent
of the above example for Retrofit):

```rust
#[macro_use] extern crate anterofit;
// Serde is also supported
extern crate rustc_serialize;


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
            // Using normal Rust expressions was easier to implement than trying to emulate 
            // Retrofit's annotation approach with attributes;
            // Plus, it's more elegant, IMO.
            GET("/posts/{}", id)
        }

        /// Create a new Post under the given user ID with the given title and body.
        fn create_post(&self, userid: u64, title: &str, body: &str) -> Post {
            POST("/posts/");
            // Provide HTTP form fields as key-value pairs
            fields! {
                "userId" => userid,
                // Shorthand for when the identifier is the same as the key
                title, 
                body
            }
        }
    }
}
```

* Construct an adapter with the base URL and serializers (unlike Retrofit, JSON serialization
is provided out-of-the-box):

```rust
use anterofit::{Adapter, Url};

let url = Url::parse("https://jsonplaceholder.typicode.com").unwrap();

let adapter = Adapter::builder()
    .base_url(url)
    // When your REST API uses JSON in both requests and responses
    .serialize_json()
    .build();
```

* Then call trait methods on the adapter, or coerce it to a trait object:

```rust
// Instead of having the adapter create opaque implementations of service traits like Retrofit,
// all service traits are implemented for `anterofit::Adapter` by default;
// using generics like this helps keep service methods namespaced.
fn create_post<T: PostService>(post_service: &T) {
    let post = post_service.new_post(42, "Hello, world!", "Lorem ipsum dolor sit amet")
        // Requests in Anterofit aren't sent until executed:
        // by default, the request will be executed via blocking I/O on a background thread.
        // This returns `Call<Post>` which can be polled on for the result.
        .exec()
        // Equivalent to `Future::wait()` without using types from `futures`
        .block()
        .unwrap();

    println!("{:?}", post);
}

// `anterofit::Adapter` is object-safe!
fn fetch_posts(post_service: &PostService) {
    let posts = post_service.get_posts()
        // Shorthand for .exec().block(), but executes the request on the current thread.
        .exec_here()
        .unwrap();

    for post in posts.into_iter().take(3) {
        println!("{:?}", post);
    }
}

create_post(&adapter);
fetch_posts(&adapter);
```

That's it. No magic, no overly complicated metaprogramming, less work being done both at compile time *and* at runtime.
On top of that, Anterofit is an order of magnitude more type-safe than Retrofit is. 
 
The `service!{}` macro is a bit of a mess mostly because of the limitations of `macro_rules!` macros, but I have plans 
to replace this using procedural macros. See the [Looking to the Future / Procedural Macros](#procedural-macros) section 
for more details.

Comparison to Retrofit
----------------------

This is kind of apples-and-oranges given that Java and Rust are two very different platforms, but I figured the 
"order of magnitude more type-safe" claim in the previous section needed substantiating.

* In Retrofit, if you pass the wrong class to `Retrofit.create()`, you get a runtime exception. 

Meainwhile in Anterofit, if you try to coerce `anterofit::Adapter` to a type that isn't a service trait, you
get a compiler error pointing right to the problem, and suggestions for types that would work.

* Serialization in Retrofit relies on a lot of magic (i.e. class metadata at runtime). Serialization for classes
doesn't have to be explicitly implemented (at least if using the GSON converter), 
so if you use the wrong type you might get surprising results.

In Anterofit, serialization for a given type has to be explicitly implemented; you'll get a compiler error otherwise. 

* In Retrofit, missing fields in deserialized objects are initialized to their defaults as per the Java language spec,
so `null` for reference types and `0` or `false` for primitives.

In Anterofit, missing fields that are not `Option` are a serialization error. Though, to be honest,
this depends on how deserialization is implemented.

Easily Construct API Wrappers
---------------------------------------

The design of Anterofit doesn't just take end-user applications into account.
 
 
Looking to the Future
---------------------

### Tokio

Once the dust around Hyper's transition to async I/O with [Tokio][tokio] settles, I plan on branching [`multipart`][multipart]
to support futures, and then rebuilding Anterofit on top of it. The public API won't change much, save for a couple new
executors which are aware of Tokio event loops; one to execute all requests on a background thread's event loop,
and one to execute requests on the event loop running in the current thread, making Anterofit truly asynchronous.

[tokio]: https://github.com/tokio-rs/tokio
[multipart]: https://github.com/abonander/multipart

### Procedural Macros