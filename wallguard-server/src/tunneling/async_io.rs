use tokio::io::{AsyncRead, AsyncWrite};

pub trait AsyncIo: AsyncRead + AsyncWrite {}
impl<T: AsyncRead + AsyncWrite> AsyncIo for T {}
