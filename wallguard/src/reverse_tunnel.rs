use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::io::Result as IoResult;
use std::{
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf},
    net::TcpStream,
};

pub struct TunnelInstance {
    pub(crate) stream: TcpStream,
}

impl AsyncRead for TunnelInstance {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for TunnelInstance {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        data: &[u8],
    ) -> Poll<IoResult<usize>> {
        Pin::new(&mut self.stream).poll_write(cx, data)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

impl From<TcpStream> for TunnelInstance {
    fn from(stream: TcpStream) -> Self {
        Self { stream }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ReverseTunnel {
    addr: SocketAddr,
}

impl ReverseTunnel {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    pub async fn request_channel(&self, token: &str) -> Result<TunnelInstance, Error> {
        let mut stream = TcpStream::connect(self.addr)
            .await
            .handle_err(location!())?;

        stream
            .write_all(token.as_bytes())
            .await
            .handle_err(location!())?;

        todo!()
    }
}
