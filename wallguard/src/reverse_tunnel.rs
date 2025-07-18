use nullnet_liberror::{location, Error, ErrorHandler, Location};
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::utilities;

#[derive(Debug, Clone, Copy)]
pub struct ReverseTunnel {
    addr: SocketAddr,
}

impl ReverseTunnel {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    pub async fn request_channel(&self, token: &str) -> Result<TcpStream, Error> {
        let hash = utilities::hash::sha256_digest_bytes(token);

        let mut stream = TcpStream::connect(self.addr)
            .await
            .handle_err(location!())?;

        stream.write_all(&hash).await.handle_err(location!())?;

        // @TODO: Confirmation message ?

        Ok(stream)
    }
}
