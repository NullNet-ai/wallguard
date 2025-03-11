use bytestring::ByteString;

pub struct PtyMessage {
    text: String,
}

impl PtyMessage {
    pub fn from_slice(buffer: &[u8]) -> Self {
        let text = String::from_utf8_lossy(buffer).to_string();
        Self { text }
    }
}

impl actix::Message for PtyMessage {
    type Result = ();
}

impl From<PtyMessage> for ByteString {
    fn from(message: PtyMessage) -> Self {
        ByteString::from(message.text)
    }
}
