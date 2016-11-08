use serialize::Serialize;

use super::{RequestAdapter, RequestAdapter_};

use mime::Mime;

type Multipart = ::multipart::client::lazy::Multipart<'static, 'static>;
type PreparedFields = ::multipart::client::lazy::PreparedFields<'static>;

use url::form_urlencoded::Serializer as UrlEncoded;

use std::fs::File;
use std::io::{self, Cursor, Read};
use std::path::PathBuf;
use std::mem;

use ::Result;

use mime::Mime;
use multipart::client::lazy::Multipart;

use std::borrow::Cow;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub trait Body: Send + 'static {
    type Readable: Read;

    fn into_readable<A>(self, adapter: &A) -> Result<Self::Readable>
    where A: RequestAdapter;
}

impl<B: Serialize + Send + 'static, A> Body for B {
    type Readable = Cursor<Vec<u8>>;

    fn into_readable<A>(self, adapter: &A) -> Result<Self::Readable>
    where A: RequestAdapter {
        let mut buf = Vec::new();

        try!(adapter.serialize(&self, &mut buf));

        Ok(Cursor::new(buf))
    }
}

impl<S> Body<S> for EmptyFields {
    type Readable = io::Empty;
    type Error = Never;

    fn into_readable(self, serializer: &S) -> Result<Self::Readable, Self::Error> {
        Ok(io::empty())
    }
}

impl<S> Body<S> for TextFields {
    type Readable = Cursor<String>;
    type Error = error::Never;

    fn into_readable(self, serializer: &S) -> Result<Self::Readable, Self::Error> where Self: Sized {

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

pub type TextFields = Vec<(String, String)>;

fn push_text_field<K: ToString, V: ToString>(text: &mut TextFields, key: K, val: V) {
    text.push((key.to_string(), val.to_string()));
}

impl Fields for TextFields {
    type WithText = Self;

    fn with_text<K: ToString, V: ToString>(mut self, key: K, val: V) -> Self {
        push_text_field(&mut self, key, val);
        self

    }

    fn with_file<K: ToString>(self, key: K, file: FileField) -> MultipartFields {
        MultipartFields::from_text(self).with_file(key, file)
    }
}

pub struct MultipartFields {
    text: TextFields,
    files: Vec<(String, FileField)>,
}

impl MultipartFields {
    fn new() -> Self {
        Self::from_text(vec![])
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
        push_text_field(&mut self.text, key, val);
        self
    }

    fn with_file<K: ToString>(mut self, key: K, file: FileField) -> MultipartFields {
        self.files.push((key.to_string(), file));
        self
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