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

/// Define the body of this request.
///
/// Can be invoked multiple times.
#[macro_export]
macro_rules! body {
    ($raw:expr) => (
        |req| req.body($raw)
    );
    ($($key:expr => $val:expr),*) => ( {
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
    ($stream:expr, filename = $filename:expr) => (
        $crate::net::FileField::from_stream($stream, Some(filename), None)
    );
    ($stream:expr, content_type: $conttype:expr) => (
        $crate::net::FileField::from_stream($stream, None, Some(content_type))
    );
    ($stream:expr, filename = $filename:expr, content_type: $conttype:expr) => (
        $crate::net::FileField::from_stream($stream, Some(filename), Some($conttype))
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