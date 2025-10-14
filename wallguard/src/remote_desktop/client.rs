use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::time::Duration;
use tokio::sync::mpsc;
use wallguard_common::timestamped_packet::TimestampedPacket;

#[derive(Debug)]
pub struct Client {
    channel: mpsc::Sender<Vec<u8>>,
}

impl Client {
    pub fn new(channel: mpsc::Sender<Vec<u8>>) -> Self {
        Self { channel }
    }

    pub async fn send(&self, data: Vec<u8>, duration: Duration) -> Result<(), Error> {
        let packet = TimestampedPacket::new(duration, data);
        self.channel
            .send(packet.to_bytes())
            .await
            .handle_err(location!())
    }
}
