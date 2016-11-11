use serialize::Serialize;

use net::adapter::{RequestAdapter, RequestAdapter_};

use mime::{self, Mime};

type Multipart = ::multipart::client::lazy::Multipart<'static, 'static>;
type PreparedFields = ::multipart::client::lazy::PreparedFields<'static>;

use url::form_urlencoded::Serializer as FormUrlEncoder;

use std::borrow::Cow;
use std::fs::File;
use std::io::{self, Cursor, Read};
use std::mem;
use std::path::PathBuf;

use ::Result;

pub type ReadableResult<T> = Result<Readable<T>>;

pub struct Readable<R: Read> {
    pub readable: R,
    pub content_type: Option<Mime>,
    // Throwaway private field for backwards compatibility.
    _private: (),
}

impl<R: Read> Readable<R> {
    /// Create a new `Readable` wrapped in `::Result::Ok` for convenience.
    pub fn new_ok<C: Into<Option<Mime>>>(readable: R, content_type: C) -> Result<Self> {
        Ok(Self::new(readable, content_type))
    }

    pub fn new<C: Into<Option<Mime>>>(readable: R, content_type: C) -> Self {
        Readable {
            readable: readable,
            content_type: content_type.into(),
            _private: (),
        }
    }
}

pub trait Body: Send + 'static {
    type Readable: Read + 'static;

    fn into_readable<A>(self, adapter: &A) -> ReadableResult<Self::Readable>
    where A: RequestAdapter;
}

impl<B: Serialize + Send + 'static> Body for B {
    type Readable = Cursor<Vec<u8>>;

    fn into_readable<A>(self, adapter: &A) -> ReadableResult<Self::Readable>
    where A: RequestAdapter {
        let mut buf = Vec::new();

        try!(adapter.serialize(&self, &mut buf));

        Readable::new_ok(Cursor::new(buf), adapter.serializer_content_type())
    }
}

/// A wrapper around a type that is intended to be read directly as the request body,
/// instead of being serialized.
pub struct RawBody<R: Read>(Readable<R>);

impl<R: Read + Send + 'static> RawBody<R> {
    pub fn new<C: Into<Option<Mime>>>(readable: R, content_type: C) -> Self {
        RawBody(Readable::new(readable, content_type))
    }
}

impl<R: AsRef<[u8]> + Send + 'static> RawBody<Cursor<R>> {
    /// Wrap anything `Cursor` can work with (such as `String` or `Vec<u8>`) as a raw request body.
    pub fn bytes(bytes: R) -> Self {
        RawBody::new(Cursor::new(bytes), mime::octet_stream())
    }
}

impl<R: Read + Send + 'static> Body for RawBody<R> {
    type Readable = R;

    fn into_readable<A>(self, adapter: &A) -> ReadableResult<Self::Readable>
    where A: RequestAdapter {
        Ok(self.0)
    }
}

pub trait Fields {
    type WithText: Fields;

    fn with_text<K: ToString, V: ToString>(self, key: K, val: V) -> Self::WithText;

    fn with_file<K: ToString>(self, key: K, file: FileField) -> MultipartFields;
}

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

    fn into_readable<A>(self, adapter: &A) -> ReadableResult<Self::Readable>
    where A: RequestAdapter {
        Readable::new_ok(io::empty(), None)
    }
}

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

    fn into_readable<A>(self, adapter: &A) -> ReadableResult<Self::Readable>
    where A: RequestAdapter {
        let readable = Cursor::new(
            FormUrlEncoder::new(String::new())
                .extend_pairs(self.0)
                .finish()
        );

        Readable::new_ok(readable, mime::form_urlencoded())
    }
}

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

    fn into_readable<A>(self, adapter: &A) -> ReadableResult<Self::Readable>
    where A: RequestAdapter {
        use self::FileField::*;

        let mut multipart = Multipart::new();

        for (key, val) in self.text.0 {
            multipart.add_text(key, val);
        }

        for (key, file) in self.files {
            match file {
                Stream {
                    stream,
                    filename,
                    content_type
                } => {
                    stream.add_self(key, filename, content_type, &mut multipart);
                },
                File(file) => {
                    // FIXME: somehow get filename and type from File, not sure if doable
                    multipart.add_stream(key, file, None as Option<String>, None);
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

enum FileField {
    Stream {
        stream: Box<StreamField>,
        filename: Option<String>,
        content_type: Option<Mime>,
    },
    File(File),
    Path(PathBuf),
}

impl FileField {
    fn from_stream<S: Read + Send + 'static>(stream: S, filename: Option<String>, content_type: Option<Mime>) -> Self {
        FileField::Stream {
            stream: Box::new(stream),
            filename: filename,
            content_type: content_type
        }
    }
}

trait StreamField: Read + Send + 'static {
    fn add_self(self: Box<Self>, name: String, filename: Option<String>, content_type: Option<Mime>, to: &mut Multipart);
}

impl<T> StreamField for T where T: Read + Send + 'static {
    fn add_self(self: Box<Self>, name: String, filename: Option<String>, content_type: Option<Mime>, to: &mut Multipart) {
        to.add_stream(name, *self, filename, content_type);
    }
}

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