/// A `try!()` macro replacement for service method bodies.
///
/// Instead of returning the error in a method returning `Result`,
/// this returns a `Request<T>` which will immediate return the error when it is invoked;
/// no network or disk activity will occur.
#[macro_export]
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
    ($adapter:expr; $method:expr; url = $($urlpart:tt)+ $(;$buildexpr:expr)*) => (
        {
            use $crate::net::RequestBuilder;

            let mut builder = RequestBuilder::new($method, url!($($urlpart)+));

            $(
                builder = ($buildexpr)(builder);
            )*

            $adapter.request(builder)
        }
    )
}

/// Serialize `$body` as the request body using the serializer provided in the adapter.
///
/// If `$body` is intended to be read directly as the request body, wrap it with `RawBody`.
///
/// This will overwrite any previous invocation of `body!()` or `fields!{}` for the current request.
#[macro_export]
macro_rules! body (
    ($body:expr) => (
        |req| req.body($body)
    );
);

/// Serialize a series of key-value pairs as the request body (form-encode them).
///
/// By default, this will serialize to a `www-form-urlencoded` body.
///
/// However, if you use the `file!()` or `stream!()` macros to define a
/// value, it will transform the request to a `multipart/form-data` request.
///
/// This will overwrite any previous invocation of `body!()` or `fields!{}` for the current request.
#[macro_export]
macro_rules! fields {
    ($($key:expr => $val:expr),*) => ({
        use $crate::net::{AddField, EmptyFields};

        let fields = $crate::net::EmptyFields;

        $(
            fields = $val.add_to($key, fields);
        )*;

        |req| req.body(fields)
    });
}

/// A field value for anything that is `Read + Send + 'static`.
///
/// Adding a stream field to the request will turn it into a `multipart/form-data` request
/// and treat it as a file field.
///
/// If given, `filename` and `content_type` keys will be supplied with the request.
/// `filename` can be a borrowed or owned string, and `content_type` should be a `Mime`
/// value from the `mime` crate.
///
/// For convenience, this crate reexports the `mime!()` macro from the `mime` crate.
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
macro_rules! file (
    ($path:expr) => (
        $crate::net::FileField::Path($path.into())
    )
);

/// Append a series of query pairs to the URL of the request.
///
/// Can be invoked multiple times.
#[macro_export]
macro_rules! query {
    ($($key:expr => $val:expr),+) => (
        |req| req.append_query(&[
            $(&$key as &::std::fmt::Display, &$val as &::std::fmt::Display),+
        ])
    )
}