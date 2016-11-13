pub use mime_::Mime;

pub fn octet_stream() -> Mime {
    mime!(Application/OctetStream)
}

pub fn json() -> Mime {
    mime!(Application/Json)
}

pub fn form_urlencoded() -> Mime {
    mime!(Application/WwwFormUrlEncoded)
}

pub fn formdata(boundary: &str) -> Mime {
    mime!(Multipart/FormData; ("boundary")=(boundary))
}

pub fn text_plain_utf8() -> Mime {
    mime!(Text/Plain; Charset=Utf8)
}

pub fn xml() -> Mime {
    mime!(Application/Xml)
}