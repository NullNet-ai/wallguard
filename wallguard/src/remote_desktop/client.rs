use nullnet_liberror::{Error, ErrorHandler, Location, location};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Client {
    channel: mpsc::Sender<Vec<u8>>,
}

impl Client {
    pub fn new(channel: mpsc::Sender<Vec<u8>>) -> Self {
        Self { channel }
    }

    pub async fn send(&self, data: Vec<u8>) -> Result<(), Error> {
        self.channel.send(data).await.handle_err(location!())
    }
}
