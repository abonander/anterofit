pub use mime_::Mime;

pub fn octet_stream() -> Mime {
    mime!(Application/Octet-Stream)
}

pub fn json() -> Mime {
    mime!(Application/Json)
}

pub fn form_urlencoded() {
    mime!(Application/WwwFormUrlEncoded)
}