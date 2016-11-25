pub use mime_::Mime;

/// `application/octet-stream`
pub fn octet_stream() -> Mime {
    mime!(Application/OctetStream)
}

/// `application/json`
pub fn json() -> Mime {
    mime!(Application/Json)
}

/// `application/www-form-urlencoded`
pub fn form_urlencoded() -> Mime {
    mime!(Application/WwwFormUrlEncoded)
}

/// `multipart/form-data; boundary={boundary}`
pub fn formdata(boundary: &str) -> Mime {
    mime!(Multipart/FormData; ("boundary")=(boundary))
}

/// `text/plain; charset=utf8`
pub fn text_plain_utf8() -> Mime {
    mime!(Text/Plain; Charset=Utf8)
}