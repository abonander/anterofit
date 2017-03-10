/// A `try!()` macro replacement for service method bodies.
///
/// Instead of returning the error in a method returning `Result`,
/// this returns a `Request<T>` which will immediate return the error when it is executed;
/// no network or disk activity will occur.
#[macro_export]
#[doc(hidden)]
macro_rules! try_request (
    ($try:expr) => (
        match $try {
            Ok(val) => val,
            Err(e) => return $crate::net::Request::immediate(Err(e.into())),
        }
    )
);

#[macro_export]
#[doc(hidden)]
macro_rules! url (
    ($urlstr:expr) => (
        $urlstr
    );
    ($urlstr:expr, $($fmt:tt)+) => (
        format!($urlstr, $($fmt)+)
    );
);

#[macro_export]
#[doc(hidden)]
macro_rules! request_impl {
    ($adapter:expr; $method:ident($($urlpart:tt)+) $(; $buildexpr:expr)*) => ({
        use $crate::net::RequestBuilder;

        let builder = RequestBuilder::new(
            $adapter, http_verb!($method), url!($($urlpart)+).into()
        );

        $(
            let builder = try_request!(builder.apply($buildexpr));
        )*

        builder.build()
    })
}

/// Allows the inside expression to set a body on a request which doesn't regularly take one.
///
/// ```rust
/// # #[macro_use] extern crate anterofit;
/// # fn main() {}
///
/// service! {
///     pub trait ForcedBodyService {
///         fn get_with_fields(&self) -> String {
///             // `GET` requests are not regularly allowed to take bodies;
///             // this is checked at compile time.
///             GET("/foo");
///             force_body! {
///                 fields! {
///                     "Hello" => "World"
///                 }
///             }
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! force_body {
    ($expr:expr) => (
        move | builder | {
            let (builder, method) = builder.swap_method(::anterofit::net::method::ForceBody);
            builder.apply($expr).map(move |builder| builder.swap_method(method).0)
        }
    )
}

/// Serialize the given value as the request body.
///
/// Serialization will be performed on the adapter's executor, using the adapter's serializer,
/// when the request is submitted.
///
/// If the value is intended to be read directly as the request body, wrap it with `RawBody`.
///
/// By default, serialization will be done on the adapter's executor,
/// so the body type must be `Send + 'static`.
///
/// If you want to serialize borrowed values or other types which cannot be sent to other threads,
/// use the `EAGER:` contextual keyword, which will cause the body to be immediately serialized
/// on the current thread.
///
/// ## Overwrites Body
/// Setting a new body will overwrite any previous body on the request.
///
/// ## Disallowed verbs: `GET, DELETE`
/// `GET` and `DELETE` requests are generally not expected to have bodies. As an
/// anti-footgun, Anterofit does not allow bodies on these requests by default.
///
/// Wrap this invocation in `force_body!()` if you want to set a body anyways.
///
/// See [`net::method::TakesBody`](net/method/trait.TakesBody.html) for more details.
#[macro_export]
macro_rules! body (
    ($body:expr) => (
        move | builder | Ok(builder.body($body))
    );
    (EAGER: $body:expr) => (
        move | builder | builder.body_eager($body)
    )
);

/// Serialize a series of key-value pairs as the request body.
///
/// The series will be serialized as if it were a map, like `HashMap` or `BTreeMap`
/// but with much less restrictive trait bounds: keys must be `Display` and values
/// must be `Serialize`; thus, keys will not be deduplicated
/// or appear in any different order than provided.
///
/// By default, serialization will be done on the adapter's executor,
/// so the key and value types must be `Send + 'static`.
///
/// If you want to serialize borrowed values or other types which cannot be sent to other threads,
/// use the `EAGER:` contextual keyword, which will cause the map to be immediately serialized on
/// the current thread.
///
/// ## Overwrites Body
/// Setting a new body will overwrite any previous body on the request.
///
/// ## Disallowed verbs: `GET, DELETE`
/// `GET` and `DELETE` requests are generally not expected to have bodies. As an
/// anti-footgun, Anterofit does not allow bodies on these requests by default.
///
/// Wrap this invocation in `force_body!()` if you want to set a body anyways.
///
/// See [`net::method::TakesBody`](net/method/trait.TakesBody.html) for more details.
#[macro_export]
macro_rules! body_map {
    ($($key:expr => $val:expr),+) => ({
        let mut pairs = $crate::serialize::PairMap::new();

        $(
            pairs.insert($key, $val);
        )+;

        move |builder| Ok(builder.body(pairs))
    });
    (EAGER: $($key:expr => $val:expr),+) => ({
        let mut pairs = $crate::serialize::PairMap::new();

        $(
            pairs.insert($key, $val);
        )+;

        move |builder| builder.body_eager(pairs)
    });
}

/// Serialize a series of fields as the request body (form-encode them).
///
/// Each field can be a key-value pair, or a single identifier. The key (field name) should be a
/// string literal, and the value can be anything that is `Display`.
///
/// For a single identifier, the identifier will be stringified for the field name, and its
/// value will become the field value:
///
/// ```rust
/// # #[macro_use] extern crate anterofit;
/// # fn main() {}
/// service! {
///     pub trait RegisterService {
///         fn register(&self, username: &str, password: &str) {
///             POST("/register");
///             fields! {
///                 "username" => username,
///                 // Equivalent to "password" => password
///                 password
///             }
///         }
///     }
/// }
/// ```
///
/// By default, this will serialize to a `www-form-urlencoded` body.
///
/// However, if you use the `path!()` or `stream!()` macros as a value expression,
/// it will transform the request to a `multipart/form-data` request.
///
/// ```rust,no_run
/// # #[macro_use] extern crate anterofit;
/// # fn main() {::anterofit::Adapter::builder().build().upload_file("file.txt".as_ref()).exec_here().unwrap()}
/// use std::path::Path;
///
/// service! {
///     pub trait UploadService {
///         fn upload_file(&self, file: &Path) {
///             POST("/upload");
///             fields! {
///                 "file" => path!(file),
///             }
///         }
///     }
/// }
/// ```
///
/// In some server stacks (e.g. PHP), these would be called `POST` parameters.
///
/// ## Overwrites Body
/// Setting a new body will overwrite any previous body on the request.
///
/// ## Disallowed verbs: `GET, DELETE`
/// `GET` and `DELETE` requests are generally not expected to have bodies. As an
/// anti-footgun, Anterofit does not allow bodies on these requests by default.
///
/// Wrap this invocation in `force_body!()` if you want to set a body anyways.
///
/// See [`net::method::TakesBody`](net/method/trait.TakesBody.html) for more details.
#[macro_export]
macro_rules! fields {
    ($($key:expr $(=> $val:expr)*),*) => ({
        use $crate::net::body::EmptyFields;

        let fields = EmptyFields;

        $(
            let fields = (field!($key, $($val)*)) (fields);
        )*;

        move |builder| Ok(builder.body(fields))
    });
    ($($key:expr $(=> $val:expr)*),*,) => ({
        use $crate::net::body::EmptyFields;

        let fields = EmptyFields;

        $(
            let fields = (field!($key, $($val)*)) (fields);
        )*;

        move |builder| Ok(builder.body(fields))
    });
}

#[doc(hidden)]
#[macro_export]
macro_rules! field {
    ($key:expr, $val:expr) => (
        move |fields| $crate::net::body::AddField::add_to($val, $key, fields)
    );
    ($keyval:expr, ) => (
        move |fields| $crate::net::body::AddField::add_to($keyval, stringify!($keyval), fields)
    )
}

/// A field value for anything that is `Read + Send + 'static`.
///
/// Adding a stream field to the request will turn it into a `multipart/form-data` request
/// and treat it as a file field.
///
/// If given, the `filename` and `content_type` keys will be supplied with the request.
/// `filename` can be a borrowed or owned string, and `content_type` should be a `Mime`
/// value from the `mime` crate.
#[macro_export]
macro_rules! stream (
    ($stream:expr) => (
        $crate::net::body::FileField::from_stream($stream, None, None)
    );
    ($stream:expr, filename: $filename:expr) => (
        $crate::net::body::FileField::from_stream($stream, Some($filename), None)
    );
    ($stream:expr, content_type: $conttype:expr) => (
        $crate::net::body::FileField::from_stream($stream, None, Some($conttype))
    );
    ($stream:expr, filename: $filename:expr, content_type: $conttype:expr) => (
        $crate::net::body::FileField::from_stream($stream, Some($filename), Some($conttype))
    );
);

/// A field value that resolves to a path on the filesystem.
///
/// The value can be anything that implements `Into<PathBuf>`, such as `&Path` or `&str`.
///
/// This will make the request into a `multipart/form-data` request and upload the file
/// that this path points to.
///
/// The filename and `Content-Type` header to be supplied with the field will be inferred from
/// the file name and extension, respectively.
///
/// To supply these values yourself, and/or your own opened file handle, see the `stream!()` macro.
#[macro_export]
macro_rules! path (
    ($path:expr) => (
        $crate::net::body::FileField::from_path($path)
    )
);

/// Append a series of query pairs to the URL of the request.
///
/// `$key` and `$val` can be anything that is `Display`; neither `Send` nor `'static` is required.
///
/// Can be invoked multiple times. Duplicate keys are left to be handled by the server.
///
/// In some server stacks (e.g. PHP), these would be called `GET` parameters.
#[macro_export]
macro_rules! query {
    ($($key:expr => $val:expr),+) => (
        |mut builder| {
            builder.head_mut().query(&[
                $((&$key as &::std::fmt::Display, &$val as &::std::fmt::Display)),+
            ]);
            Ok(builder)
        }
    )
}

/// Use in a service method body to perform an arbitrary transformation on the builder.
///
/// ```rust
/// # #[macro_use] extern crate anterofit;
/// # fn main() {}
/// use anterofit::RawBody;
///
/// service! {
///     trait MyService {
///         fn send_whatever(&self) {
///             POST("/whatever");
///             // `move` and `mut` are allowed in their expected positions as well
///             map_builder!(|builder| builder.body(RawBody::text("Hello, world!")))
///         }
///     }
/// }
/// ```
///
/// You can even use `try!()` as long as the error type is convertible to `anterofit::Error`:
///
/// ```rust
/// # #[macro_use] extern crate anterofit;
/// # fn main() {}
/// use anterofit::RawBody;
/// use std::fs::File;
///
/// service! {
///     trait MyService {
///         fn put_log_file(&self) {
///             PUT("/log");
///             map_builder!(|builder| {
///                 let logfile = try!(File::open("/etc/log"));
///                 builder.body(RawBody::new(logfile, None))
///             })
///         }
///     }
/// }
/// ```
///
/// If you just want to return a `Result<RequestBuilder>`, use a bare closure in your service
/// method body.
#[macro_export]
macro_rules! map_builder {
    (|$builder:ident| $expr:expr) => (
        |$builder| Ok($expr)
    );
    (move |$builder:ident| $expr:expr) => (
        move |$builder| Ok($expr)
    );
    (|mut $builder:ident| $expr:expr) => (
        |mut $builder| Ok($expr)
    );
    (move |mut $builder:ident| $expr:expr) => (
        move |mut $builder| Ok($expr)
    );
}

/// Use in a service body to access the builder without consuming it.
///
/// The expression can resolve to anything, as the result is silently discarded.
///
/// ```rust
/// # #[macro_use] extern crate anterofit;
/// # fn main() {}
/// service! {
///     trait MyService {
///         fn get_whatever(&self) {
///             GET("/whatever");
///             with_builder!(|builder| println!("Request: {:?}", builder.head()))
///         }
///     }
/// }
/// ```
///
/// You can even use `try!()` as long as the error type is convertible to `anterofit::Error`:
///
/// ```rust,no_run
/// # #[macro_use] extern crate anterofit;
/// # fn main() {}
/// use std::fs::OpenOptions;
/// // Required for `write!()`
/// use std::io::Write;
///
/// service! {
///     trait MyService {
///         fn get_whatever(&self) {
///             GET("/whatever");
///             with_builder!(|builder| {
///                 let mut logfile = try!(OpenOptions::new()
///                     .append(true).create(true).open("/etc/log"));
///                 try!(write!(logfile, "Request: {:?}", builder.head()));
///             })
///         }
///     }
/// }
/// ```
///
/// (In practice, logging requests should probably be done in an `Interceptor` instead;
/// this is merely an example demonstrating a plausible use-case.)
#[macro_export]
macro_rules! with_builder {
    (|$builder:ident| $expr:expr) => (
        |$builder| {
            let _ = $expr;
            Ok($builder)
        }
    );
    (move |$builder:ident| $expr:expr) => (
        move |$builder {
            let _ = $expr;
            Ok($builder)
        }
    );
    (|mut $builder:ident| $expr:expr) => (
        |mut $builder| {
            let _ = $expr;
            Ok($builder)
        }
    );
    (move |mut $builder:ident| $expr:expr) => (
        |mut $builder| {
            let _ = $expr;
            Ok($builder)
        }
    );
}

#[doc(hidden)]
#[macro_export]
macro_rules! http_verb {
    (GET) => ($crate::net::method::Get);
    (POST) => ($crate::net::method::Post);
    (PUT) => ($crate::net::method::Put);
    (PATCH) => ($crate::net::method::Patch);
    (DELETE) => ($crate::net::method::Delete);
}