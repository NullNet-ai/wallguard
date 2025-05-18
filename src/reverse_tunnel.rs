use nullnet_liberror::{location, Error, ErrorHandler, Location};
use sha2::Digest;
use sha2::Sha256;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

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
        let mut hash = token_digest(token);

        let mut stream = TcpStream::connect(self.addr)
            .await
            .handle_err(location!())?;

        stream.write_all(&mut hash).await.handle_err(location!())?;

        // @TODO: Confirmation message ?

        Ok(stream)
    }
}

fn token_digest(token: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    result.as_slice().try_into().unwrap()
}
