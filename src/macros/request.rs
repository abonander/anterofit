/// A `try!()` macro replacement for service method bodies.
///
/// Instead of returning the error in a method returning `Result`,
/// this returns a `Request<T>` which will immediate return the error when it is executed;
/// no network or disk activity will occur.
#[macro_export]
#[doc(hidden)]
macro_rules! try_request (
    ($adpt:expr, $try:expr) => (
        match $try {
            Ok(val) => val,
            Err(e) => return $crate::net::Request::immediate($adpt, Err(e.into())),
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
    ($adapter:ident; $method:expr; url($($urlpart:tt)+) $(; $buildexpr:expr)*) => ({
        use $crate::net::RequestBuilder;

        let builder = RequestBuilder::new($adapter, $method, url!($($urlpart)+).into());

        $(
            let builder = try_request!($adapter, ($buildexpr)(builder));
        )*

        builder.build()
    })
}

/// Serialize the given value as the request body using the serializer provided in the adapter.
///
/// If the value is intended to be read directly as the request body, wrap it with `RawBody`.
///
/// This will overwrite any previous invocation of `body!()` or `fields!{}` for the current request.
///
/// ## Panics
/// If the request is a GET request (cannot have a body).
#[macro_export]
macro_rules! body (
    ($body:expr) => (
        // UFCS is necessary as the compiler can't infer the type otherwise
        move | req | Ok($crate::net::RequestBuilder::body(req, $body))
    )
);

/// Like `body!()`, but eagerly serializes the body on the current thread.
///
/// This is useful when you have a request body that is not `Send + 'static`.
#[macro_export]
macro_rules! body_eager (
    ($body:expr) => (
        move | req | $crate::net::RequestBuilder::body_eager(req, $body)
    );
);

/// Serialize a series of key-value pairs as the request body (form-encode them).
///
/// If passed a list of identifiers
///
/// By default, this will serialize to a `www-form-urlencoded` body.
///
/// However, if you use the `path!()` or `stream!()` macros to define a
/// value, it will transform the request to a `multipart/form-data` request.
///
/// This will overwrite any previous invocation of `body!()` or `fields!{}` for the current request.
///
/// ##Example
/// ```rust,ignore
/// fields! {
///     // Use key-value pair
///     "username" => username,
///     // Stringify the identifier as the key, use its value as the value
///     password
/// }
/// ```
///
/// ## Panics
/// If the request is a GET request (cannot have a body).
#[macro_export]
macro_rules! fields {
    ($($key:expr $(=> $val:expr)*),*) => ({
        use $crate::net::{AddField, EmptyFields};

        let fields = $crate::net::EmptyFields;

        $(
            fields = (field!($key, $($val)*)) (fields);
        )*;

        move |req| Ok($crate::net::RequestBuilder::body(req, fields))
    })
}

#[doc(hidden)]
#[macro_export]
macro_rules! field {
    ($key:expr, $val:expr) => (
        move |fields| $crate::net::AddField::add_to($val, $key, fields)
    );
    ($keyval:expr, ) => (
        move |fields| $crate::net::AddField::add_to($keyval, stringify!($keyval), fields)
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
        $crate::net::FileField::from_stream($stream, None, None)
    );
    ($stream:expr, filename: $filename:expr) => (
        $crate::net::FileField::from_stream($stream, Some($filename), None)
    );
    ($stream:expr, content_type: $conttype:expr) => (
        $crate::net::FileField::from_stream($stream, None, Some($conttype))
    );
    ($stream:expr, filename: $filename:expr, content_type: $conttype:expr) => (
        $crate::net::FileField::from_stream($stream, Some($filename), Some($conttype))
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
/// the file name and extension, respectably.
///
/// To supply these values yourself, and/or your own opened file handle, see the `stream!()` macro.
#[macro_export]
macro_rules! path (
    ($path:expr) => (
        $crate::net::FileField::from_path($path)
    )
);

/// Append a series of query pairs to the URL of the request.
///
/// Can be invoked multiple times.
#[macro_export]
macro_rules! query {
    ($($key:expr => $val:expr),+) => (
        |req| Ok($crate::net::RequestBuilder::query(req, &[
            $(&$key as &::std::fmt::Display, &$val as &::std::fmt::Display),+
        ]))
    )
}