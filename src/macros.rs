#[macro_export]
macro_rules! service {
    (
        pub trait $servicenm:ident {
            $($methods:tt)*
        }
    ) => (
        pub trait $servicenm : $crate::net::RequestAdapter {
            $($methods)*
        }

        impl<T: $crate::net::RequestAdapter> $servicenm for T {}
    )
}

#[macro_export]
macro_rules! get {
    (
        $(#[$meta:meta])*
        fn $fnname:ident ($($args:tt)*) -> $ret:ty {
                $($body:tt)+
        }
    ) => (
        $(#[$meta])*
        fn $fnname (&self, $($args)*) -> $crate::Request<Self, $ret> {
            request_impl! {
                self; $crate::net::Method::Get;
                $($body)+
            }
        }
    );
    (
        $(#[$meta:meta])*
        fn $fnname:ident <$($generics:tt)*> ($($arg:pat),+) -> $ret:ty {
                $($body:tt)+
        }
    ) => (
        $(#[$meta])*
        fn $fnname <$($generics)*> (&self, $($arg),+) -> $crate::Request<Self, $ret> {
            request_impl! {
                self; $crate::net::Method::Get;
                $($body)+
            }
        }
    );
}

#[macro_export]
#[doc(hidden)]
macro_rules! request_impl {
    ($adapter:expr; $method:expr; url = $($urlpart:tt)+ $(;$buildexpr:expr)*) => (
        {
            use $crate::net::RequestBuilder;

            let url = format!($($urlpart)+);

            let mut builder = RequestBuilder::new($method, &url);

            $(
                builder = ($buildexpr)(builder);
            )*

            self.request(builder)
        }
    )
}

/// Define the body of this request.
///
/// Can be invoked multiple times.
macro_rules! body {
    ($raw:expr) => (
        |req| req.body($raw)
    );
    ($($key:expr => $val:expr),*) => (
        let mut fields = $crate::net::Fields::Empty;

        $(
            fields.add_field($key, $val);
        )*;

        fields
    );
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
        |req3| req.append_query(&[
            $(&$key as &::std::fmt::Display, &$val as &::std::fmt::Display),+
        ])
    )
}