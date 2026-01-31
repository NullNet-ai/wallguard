use std::io::Result;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct TunnelInstance {
    pub(super) stream: TcpStream,
}

impl From<TcpStream> for TunnelInstance {
    fn from(stream: TcpStream) -> Self {
        Self { stream }
    }
}

impl AsyncRead for TunnelInstance {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for TunnelInstance {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        data: &[u8],
    ) -> Poll<Result<usize>> {
        Pin::new(&mut self.stream).poll_write(cx, data)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

impl TunnelInstance {
    pub async fn shutdown(&mut self) -> Result<()> {
        use tokio::io::AsyncWriteExt;
        self.stream.shutdown().await
    }

    pub fn take_stream(self) -> TcpStream {
        self.stream
    }
}
