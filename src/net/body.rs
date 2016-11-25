//! Types that can be serialized to bodies of HTTP requests.

use serialize::{Serialize, Serializer};

use net::adapter::RequestAdapter;

use mime::{self, Mime};

type Multipart = ::multipart::client::lazy::Multipart<'static, 'static>;
type PreparedFields = ::multipart::client::lazy::PreparedFields<'static>;

use url::form_urlencoded::Serializer as FormUrlEncoder;

use std::borrow::Borrow;
use std::io::{self, Cursor, Read};
use std::path::PathBuf;

use ::Result;

/// The result type for `Body::into_readable()`.
pub type ReadableResult<T> = Result<Readable<T>>;

/// The result of serializing the request body, ready to be sent over the network.
#[derive(Debug)]
pub struct Readable<R> {
    /// The inner `Read` impl which will be copied into the request body.
    pub readable: R,
    /// The MIME type of the request body, if applicable.
    pub content_type: Option<Mime>,
    // Throwaway private field for backwards compatibility.
    _private: (),
}

impl<R: Read> Readable<R> {
    /// Create a new `Readable` wrapped in `::Result::Ok` for convenience.
    pub fn new_ok<C: Into<Option<Mime>>>(readable: R, content_type: C) -> Result<Self> {
        Ok(Self::new(readable, content_type))
    }

    /// Create a new `Readable` with the given `Read` and MIME type (can be an `Option` or a bare
    /// `Mime` value).
    pub fn new<C: Into<Option<Mime>>>(readable: R, content_type: C) -> Self {
        Readable {
            readable: readable,
            content_type: content_type.into(),
            _private: (),
        }
    }
}

/// A trait describing a type which can be serialized into a request body.
///
/// Implemented for `T: Serialize + Send + 'static`.
pub trait Body: Send + 'static {
    /// The readable request body.
    type Readable: Read + 'static;

    /// Serialize `self` with the given adapter into a request body.
    fn into_readable<A>(self, adapter: &A) -> ReadableResult<Self::Readable>
    where A: RequestAdapter;
}

impl<B: EagerBody + Send + 'static> Body for B {
    type Readable = <B as EagerBody>::Readable;

    fn into_readable<A>(self, adapter: &A) -> ReadableResult<Self::Readable> where A: RequestAdapter {
        <B as EagerBody>::into_readable(self, adapter)
    }
}

/// A trait describing a type which can be serialized into a request body.
///
/// Implemented for `T: Serialize + Send + 'static`.
pub trait EagerBody {
    /// The readable request body.
    type Readable: Read + Send + 'static;

    /// Serialize `self` with the given adapter into a request body.
    fn into_readable<A>(self, adapter: &A) -> ReadableResult<Self::Readable>
        where A: RequestAdapter;
}

impl<B: Serialize> EagerBody for B {
    type Readable = Cursor<Vec<u8>>;

    fn into_readable<A>(self, adapter: &A) -> ReadableResult<Self::Readable>
        where A: RequestAdapter {
        let mut buf = Vec::new();

        let serializer = adapter.serializer();

        try!(serializer.serialize(&self, &mut buf));

        Readable::new_ok(Cursor::new(buf), serializer.content_type())
    }
}

/// A wrapper around a type that is intended to be read directly as the request body,
/// instead of being serialized.
#[derive(Debug)]
pub struct RawBody<R>(Readable<R>);

impl<R: Read> RawBody<R> {
    /// Wrap a `Read` type and a content-type
    pub fn new<C: Into<Option<Mime>>>(readable: R, content_type: C) -> Self {
        RawBody(Readable::new(readable, content_type))
    }
}

impl<R: AsRef<[u8]>> RawBody<Cursor<R>> {
    /// Wrap anything `Cursor` can work with (such as `String` or `Vec<u8>`) as a raw request body.
    ///
    /// Assumes `application/octet-stream` as the content-type.
    pub fn bytes(bytes: R) -> Self {
        RawBody::new(Cursor::new(bytes), mime::octet_stream())
    }
}

impl RawBody<Cursor<String>> {
    /// Convert the `ToString` value to a `String` and wrap it.
    ///
    /// Assumes `text/plain; charset=utf8` as the content-type.
    pub fn display<T: ToString>(text: &T) -> Self {
        RawBody::text(text.to_string())
    }
}

impl RawBody<Cursor<Vec<u8>>> {
    /// Use the serializer in `adapter` to serialize `val` as a raw body immediately.
    pub fn serialize_now<A, T>(adapter: &A, val: &T) -> Result<Self>
    where A: RequestAdapter, T: Serialize {
        let mut buf: Vec<u8> = Vec::new();
        let serializer = adapter.serializer();
        try!(serializer.serialize(val, &mut buf));
        Ok(RawBody::new(Cursor::new(buf), serializer.content_type()))
    }
}

impl<T: Borrow<str> + AsRef<[u8]>> RawBody<Cursor<T>> {
    /// Wrap anything `Send + 'static` that can deref to `str`
    /// (`String`, `&'static str`, `Box<str>`, etc)
    /// as a plain text body.
    ///
    /// Assumes `text/plain; charset=utf8` as the content-type.
    pub fn text(text: T) -> Self {
        RawBody::new(Cursor::new(text), mime::text_plain_utf8())
    }
}

impl<R: Read + Send + 'static> Body for RawBody<R> {
    type Readable = R;

    fn into_readable<A>(self, _adapter: &A) -> ReadableResult<Self::Readable>
    where A: RequestAdapter {
        Ok(self.0)
    }
}

impl<R> From<Readable<R>> for RawBody<R> {
    fn from(readable: Readable<R>) -> Self {
        RawBody(readable)
    }
}

/// Helps save some imports and typing.
pub type RawBytesBody = RawBody<Cursor<Vec<u8>>>;

/// A builder trait describing collections of key-value pairs to be serialized into a request body.
pub trait Fields {
    /// The type that results from adding a text field; may or may not change depending on the
    /// initial type.
    type WithText: Fields;

    /// Add a key-text value pair to this fields collection, returning the resulting type.
    fn with_text<K: ToString, V: ToString>(self, key: K, val: V) -> Self::WithText;

    /// Add a key-file vale pair to this fields collection, returning the resulting type.
    fn with_file<K: ToString>(self, key: K, file: FileField) -> MultipartFields;
}

/// An empty fields collection, will serialize to nothing.
pub struct EmptyFields;

impl Fields for EmptyFields {
    type WithText = TextFields;

    fn with_text<K: ToString, V: ToString>(self, key: K, val: V) -> TextFields {
        TextFields::new().with_text(key, val)
    }

    fn with_file<K: ToString>(self, key: K, file: FileField) -> MultipartFields {
        MultipartFields::new().with_file(key, file)
    }
}

impl Body for EmptyFields {
    type Readable = io::Empty;

    fn into_readable<A>(self, _adapter: &A) -> ReadableResult<Self::Readable>
    where A: RequestAdapter {
        Readable::new_ok(io::empty(), None)
    }
}

/// A collection of key-string value pairs to be serialized as fields in the request.
///
/// Will be serialized as form/percent-encoded pairs.
pub struct TextFields(Vec<(String, String)>);

impl TextFields {
    fn new() -> TextFields {
        TextFields(vec![])
    }

    fn push<K: ToString, V: ToString>(&mut self, key: K, val: V) {
        self.0.push((key.to_string(), val.to_string()));
    }
}

impl Fields for TextFields {
    type WithText = Self;

    fn with_text<K: ToString, V: ToString>(mut self, key: K, val: V) -> Self {
        self.push(key, val);
        self
    }

    fn with_file<K: ToString>(self, key: K, file: FileField) -> MultipartFields {
        MultipartFields::from_text(self).with_file(key, file)
    }
}

impl Body for TextFields {
    type Readable = Cursor<String>;

    fn into_readable<A>(self, _adapter: &A) -> ReadableResult<Self::Readable>
    where A: RequestAdapter {
        let readable = Cursor::new(
            FormUrlEncoder::new(String::new())
                .extend_pairs(self.0)
                .finish()
        );

        Readable::new_ok(readable, mime::form_urlencoded())
    }
}

/// A collection of key-value pairs where the values may be string fields or file fields.
///
/// Will be serialized as a `multipart/form-data` request.
pub struct MultipartFields {
    text: TextFields,
    files: Vec<(String, FileField)>,
}

impl MultipartFields {
    fn new() -> Self {
        Self::from_text(TextFields::new())
    }

    fn from_text(text: TextFields) -> Self {
        MultipartFields {
            text: text,
            files: vec![],
        }
    }
}

impl Fields for MultipartFields {
    type WithText = Self;

    fn with_text<K: ToString, V: ToString>(mut self, key: K, val: V) -> Self::WithText {
        self.text.push(key, val);
        self
    }

    fn with_file<K: ToString>(mut self, key: K, file: FileField) -> MultipartFields {
        self.files.push((key.to_string(), file));
        self
    }
}

impl Body for MultipartFields {
    type Readable = PreparedFields;

    fn into_readable<A>(self, _adapter: &A) -> ReadableResult<Self::Readable>
    where A: RequestAdapter {
        use self::FileField_::*;

        let mut multipart = Multipart::new();

        for (key, val) in self.text.0 {
            multipart.add_text(key, val);
        }

        for (key, file) in self.files {
            match file.0 {
                Stream {
                    stream,
                    filename,
                    content_type
                } => {
                    stream.add_self(key, filename, content_type, &mut multipart);
                },
                Path(path) => {
                    multipart.add_file(key, path);
                }
            }
        }

        let prepared = try!(multipart.prepare());

        let content_type = mime::formdata(prepared.boundary());

        Readable::new_ok(prepared, content_type)
    }
}


/// A file field, can be a generic `Read` impl or a `Path`.
pub struct FileField(FileField_);

impl FileField {
    /// Wrap a `Read` impl with an optional filename and MIME type to be serialized as a file field.
    pub fn from_stream<S: Read + Send + 'static>(stream: S, filename: Option<String>, content_type: Option<Mime>) -> Self {
        FileField(FileField_::Stream {
            stream: Box::new(stream),
            filename: filename,
            content_type: content_type
        })
    }

    /// Wrap a `Path` to be serialized as a file field, inferring its filename and MIME type.
    pub fn from_path<P: Into<PathBuf>>(path: P) -> Self {
        FileField(FileField_::Path(path.into()))
    }
}

enum FileField_ {
    Stream {
        stream: Box<StreamField>,
        filename: Option<String>,
        content_type: Option<Mime>,
    },
    Path(PathBuf),
}

trait StreamField: Read + Send + 'static {
    fn add_self(self: Box<Self>, name: String, filename: Option<String>, content_type: Option<Mime>, to: &mut Multipart);
}

impl<T> StreamField for T where T: Read + Send + 'static {
    fn add_self(self: Box<Self>, name: String, filename: Option<String>, content_type: Option<Mime>, to: &mut Multipart) {
        to.add_stream(name, *self, filename, content_type);
    }
}

#[doc(hidden)]
pub trait AddField<F> {
    type Output: Fields;

    fn add_to<K: ToString>(self, key: K, to: F) -> Self::Output;
}

impl<F: Fields, T: ToString> AddField<F> for T {
    type Output = <F as Fields>::WithText;

    fn add_to<K: ToString>(self, key: K, to: F) -> F::WithText {
        to.with_text(key, self)
    }
}

impl<F: Fields> AddField<F> for FileField {
    type Output = MultipartFields;

    fn add_to<K: ToString>(self, key: K, to: F) -> MultipartFields {
        to.with_file(key, self)
    }
}