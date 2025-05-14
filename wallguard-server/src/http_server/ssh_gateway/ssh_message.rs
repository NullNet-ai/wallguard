pub(super) struct SSHMessage {
    pub(super) data: Vec<u8>,
}

impl From<&[u8]> for SSHMessage {
    fn from(value: &[u8]) -> Self {
        SSHMessage {
            data: Vec::from(value),
        }
    }
}

impl actix::Message for SSHMessage {
    type Result = ();
}
