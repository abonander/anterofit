Anterofit Release Announcement
================

The common practice when releasing any relatively novel crate is  to write a blog post
about the subject. However, since I don't have a blog set up (and, to be brutally honest,
I find the idea of having a blog somewhat pretentious), I figured I'd just write the release
announcement in the repo itself.

Anterofit has been a long time coming. I've been conceptualizing it ever since I started tinkering with Rust,
something like over three years ago. For the longest time, I thought it wasn't possible without compiler
plugins, which dampened my enthusiasm somewhat as they are perpetually unstable and break occasionally without much 
warning. As it turns out, I was partially right, as the macro-based implementation has
one major ugly limitation, but the future is looking bright.

Inspiration
-----------

I basically straight-up copied the concept behind [Retrofit][retrofit], as a
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

For a useful example, the interface and usage to fetch a post from [the JSONPlaceholder API][jsonplaceholder], 
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
 
Introducing Anterofit
---------------------

Anterofit is a collection of macros that make it easy to wrap REST APIs using Rust traits. 

The name didn't really sound "cool" when I first came up with it, but it's grown on me. 


[retrofit]: http://square.github.io/retrofit/