Anterofit User's Guide
======================

Anterofit makes it easy to abstract over REST APIs and asynchronous requests.

The core of Anterofit's abstraction power lies in its macro-based
generation of service traits, eliminating the noisy boilerplate
involved in creating and issuing HTTP requests. The
focal point of this power lies in the `service!{}` macro.

See the [README](README.md) for setting up dependencies and choosing a serialization framework.

####Note
This document is a work-in-progress. If there is any information which you think would be helpful
to add, please feel free to open a pull request.

The [API Documentation](https://docs.rs/anterofit) also contains a wealth of information about
Anterofit and its functionality.

Creating Service Traits with `service!{}`
-----------------------------------------

`service!{}` simply takes the definition of a trait item, with its method bodies in a particular format,
and generates an object-safe implementation of the service trait for the `Adapter` type.

```rust
service! {
    /// Trait wrapping `myservice.com` API.
    pub trait MyService {
        /// Get the version of this API.
        fn api_version(&self) -> String {
            GET("/version")
        }

        /// Register a user with the API.
        fn register(&self, username: &str, password: &str) {
            POST("/register");
            fields! {
                username, password
            }
        }
    }
}
```

#### Service Trait Method Overview

Service trait methods always take `&self` as the first parameter; this is purely
an implementation detail. Method parameters are passed to the implementation, unchanged.
They can be borrowed or owned, but there is some restriction on the usage
of borrowed parameters.

The return type of a trait method can be any type that implements the correct deserialization trait
for the serialization framework you're using:

* `rustc-serialize`: `Decodable`, derived with `#[derive(RustcDecodable)]`

* Serde: `Deserialize`, derived with `#[derive(Deserialize)]` via either 
[`serde_codegen`](https://crates.io/crates/serde_codegen) (build script) or 
[`serde_derive`](https://crates.io/crates/serde_derive) (procedural macro).

The return type can also be omitted. Just like in regular Rust, it is implied to be `()`.

The body of a trait method is syntactically the same
as any (non-empty) Rust function: zero or more semicolon-terminated statements/expressions 
followed by an unterminated expression. However, there are a couple of major differences:

#### HTTP Verb and URL

The first expression, which is always required, is structured like a function call, where the identifier 
outside is an HTTP verb and the inside is the URL string and any optional formatting arguments, in the vein
of `format!()` or `println!()`. This allows parameters to be interpolated into the URL. 
The most common HTTP verbs are supported: `GET POST PUT PATCH DELETE`

```
// If `id` is some parameter that implements `Display`
GET("/version")
GET("/posts/{}", id)
POST("/posts/update/{}", id)
DELETE("/posts/{id}", id=id)
```

Notice that the paths in these declarations are not assumed to be complete URLs; instead, they will be appended to the 
base URL provided in the adapter. However, if necessary, they *can* be complete URLs, with the base URL being omitted 
during the construction of the adapter.

#### Request Modifiers (query pairs, form fields, etc)

All expressions following the first, if any, are treated as modifiers to the request. 
Syntactically, any expression is allowed, but arbitrary expressions will likely not typecheck due to some
implementation details of the `service!{}` macro. Instead, you are expected to use the other macros provided 
by Anterofit to modify the request.

See the [`Macros` header in the crate docs][doc-macros] for more information.

* To add query parameters, sometimes called `GET` parameters, use `query!{}`
 
* To add form fields, sometimes called `POST` parameters, use `fields!{}`.
 You can see this being used in the first example.
 
    * To add a file to be uploaded, use `path!()` (takes anything convertible to `PathBuf`) as a field value.
    
    * To add a stream to be uploaded (can be any generic `Read` impl), use `stream!()` as a field value.
    
* To set the request body, use the `body!()` macro. You would use this if your REST API is expecting parameters 
passed as, e.g. JSON, instead of in an HTTP form; see the [Serialization](#serialization) header for more information.

* To set the request body as a series of key-value pairs, use `body_map!()`. This behaves as if you passed
a `HashMap` or `BTreeMap` of the key-value pairs to `body!()`, but does not require the keys to implement 
any trait except `std::fmt::Display` (thus, keys are not deduplicated or reordered--the server is expected to handle
it); values are, of course, expected to implement the serialization trait from the serialization framework you're using.

* To apply arbitrary mutations or transformations to the request builder, use `with_builder!()` or `map_builder!()`, 
respectively.

For more advanced usage, you can use bare closure expressions that take `RequestBuilder` and return 
`Result<RequestBuilder, anterofit::Error>`. See `RequestBuilder::apply()`, which is used as a type hint
so that no type annotations are required on the closures (very convenient, by the way). All the aforementioned
macros wrap this mechanism.

[doc-macros]: http://docs.rs/anterofit#macros

#### Delegation

When using Anterofit in a library context, such as when writing a wrapper for a public REST API, like Reddit's or Github's,
you may want to control construction of and access to the `Adapter` to limit potential footguns, but you may still
 want to use service traits in your public API to limit duplication. 
 
 By default, service traits are implemented for 
 `Adapter`, so this may seem at odds with the desire for abstraction. However, you can override this and have Anterofit automatically generate implementations of your service
 traits for a custom type: all that is required is a closure expression that will serve as an accessor for the 
 inner `Adapter` instance: 

```rust
pub struct MyDelegate {
    // `Adapter`'s type parameters omitted for brevity
    adapter: ::anterofit::Adapter<...>,
}

service! {    
    pub trait MyService {
        /// Get the version of this API.
        fn api_version(&self) -> String {
            GET("/version")
        }

        /// Register a user with the API.
        fn register(&self, username: &str, password: &str) {
            POST("/register");
            fields! {
                username, password
            }
        }    
    }
    
    // The expression inside the braces is expected to be `FnOnce(&Self) -> &Adapter<...>`
    impl for MyDelegate { |this| &this.adapter }
}
```

Notice that the adapter is completely concealed inside `MyDelegate`, but because of Rust's visibility
 rules, the service trait's impl can still access it. 

Serialization
-------------

Anterofit supports both serialization of request bodies, and deserialization of response bodies. However,
Anterofit does not use any specified data format by default. The default serializer returns an error for all types,
and the default deserializer only supports primitives and strings.

If you want to use the `body!()` or `body_map!()` macros in a request method, you'll need to set
a `Serializer` during construction of the adapter. Similarly, if you want to deserialize responses as complex types, 
you'll need to set a `Deserializer` at the same time.

For serializing and deserializing JSON, the adapter builder has a convenience method: `serialize_json()`,
and the `JsonAdapter` typedef for ease of naming.

As of January 2017, Anterofit supports JSON serialization and deserialization *only*.

Relevant types are in the `serialize` module.

