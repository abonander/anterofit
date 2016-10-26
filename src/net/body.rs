use serialize::{Serializer, Deserializer, Serialize, Deserialize};

use mime::Mime;

type Multipart = ::multipart::client::lazy::Multipart<'static, 'static>;
type PreparedFields = ::multipart::client::lazy::PreparedFields<'static>;

use url::form_urlencoded::Serializer as UrlEncoded;

use std::fs::File;
use std::io::{self, Cursor, Read};
use std::path::PathBuf;
use std::mem;

type TextFields = Vec<(String, String)>;
type FileFields = Vec<(String, FileField)>;

pub enum Fields {
    Empty,
    Text(TextFields),
    Multipart {
        texts: TextFields,
        files: FileFields,
    },
}

impl Fields {

    pub fn add_field<K: ToString, F: AddField>(&mut self, key: K, field: F) {
        field.add_to(key, self);
    }

    fn push_file_field<K: ToString>(&mut self, key: K, val: FileField) {
        self.files_mut().push((key.to_string(), val))
    }

    fn texts_mut(&mut self) -> &mut TextFields {
        if let Fields::Empty = *self {
            *self = Fields::Text(vec![]);
        }

        match *self {
            Fields::Text(ref mut texts) => texts,
            Fields::Multipart { ref mut texts, .. } => texts,
            Fields::Empty => unreachable!(),
        }
    }

    fn files_mut(&mut self) -> &mut FileFields {
        if let Fields::Multipart { ref mut files, .. } = *self {
            return files;
        }

        let mut multipart = Fields::Multipart {
            texts: mem::replace(self.texts_mut(), vec![]),
            files: vec![],
        };

        *self = multipart;

        if let Fields::Multipart { ref mut files, .. } = *self {
            return files;
        }

        unreachable!();
    }
}

pub trait Body<S>: Send + 'static {
    type Readable: Read + 'static;
    type Error;

    fn into_readable(self, serializer: &S) -> Result<Self::Readable, Self::Error>;
}

impl<B: Serialize + Send + 'static, S: Serializer> Body<S> for B {
    type Readable = Cursor<Vec<u8>>;
    type Error = <S as Serializer>::Error;

    fn into_readable(self, serializer: &S) -> Result<Self::Readable, Self::Error> {
        let mut buf = Vec::new();

        try!(serializer.serialize(&self, &mut buf));

        Ok(Cursor::new(buf))
    }
}

pub enum FileField {
    Stream {
        stream: Box<StreamField>,
        filename: Option<String>,
        content_type: Option<Mime>,
    },
    File(File),
    Path(PathBuf),
}

impl FileField {
    pub fn from_stream<S: Read + Send + 'static>(stream: S, filename: Option<String>, content_type: Option<Mime>) -> Self {
        FileField::Stream {
            stream: Box::new(stream),
            filename: filename,
            content_type: content_type
        }
    }
}

pub trait StreamField: Read + Send + 'static {
    fn add_self(self: Box<Self>, name: String, filename: Option<String>, content_type: Option<Mime>, to: &mut Multipart);
}

impl<T> StreamField for T where T: Read + Send + 'static {
    fn add_self(self: Box<Self>, name: String, filename: Option<String>, content_type: Option<Mime>, to: &mut Multipart) {
        to.add_stream(name, *self, filename, content_type);
    }
}

pub trait AddField {
    fn add_to<K: ToString>(self, key: K, to: &mut Fields);
}

impl<T: ToString> AddField for T {
    fn add_to<K: ToString>(self, key: K, to: &mut Fields) {
        to.texts_mut().push((key.to_string(), self.to_string()));
    }
}

impl AddField for FileField {
    fn add_to<K: ToString>(self, key: K, to: &mut Fields) {
        to.push_file_field(key, self);
    }
}