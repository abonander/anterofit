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
/// By default, this will serialize to a `www-form-urlencoded` body.'
///
/// However, if you use the `file!()`, `path!()`, or `stream!()` macros to define a
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

#[macro_export]
macro_rules! stream {
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
}

#[macro_export]
macro_rules! file (
    ($file:expr) => (
        $crate::net::FileField::File(file)
    )
);

/// A field value that resolves to a path on the filesystem.
///
/// This will make the request into a `multipart/form-data` request
/// and upload the file that this path points to.
#[macro_export]
macro_rules! path (
    ($path:expr) => (
        $crate::net::FileField::Path($path.into())
    )
);

/// Set the query of this request.
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