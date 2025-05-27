use nullnet_liberror::{location, Error, ErrorHandler, Location};
use sha2::Digest;
use sha2::Sha256;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::utilities;

#[derive(Debug, Clone, Copy)]
pub struct ReverseTunnel {
    addr: SocketAddr,
}

impl ReverseTunnel {
    pub fn new(addr: &str, port: u16) -> Result<Self, Error> {
        let addr = format!("{}:{}", addr, port)
            .parse()
            .handle_err(location!())?;

        Ok(Self { addr })
    }

    pub async fn request_channel(&self, token: &str) -> Result<TcpStream, Error> {
        let mut hash = utilities::hash::sha256_digest_bytes(token);

        let mut stream = TcpStream::connect(self.addr)
            .await
            .handle_err(location!())?;

        stream.write_all(&mut hash).await.handle_err(location!())?;

        // @TODO: Confirmation message ?

        Ok(stream)
    }
}