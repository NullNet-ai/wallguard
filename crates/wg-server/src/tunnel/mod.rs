pub mod listener;
pub mod registry;

pub use registry::TunnelRegistry;

use tokio::io::{AsyncRead, AsyncWrite};

pub struct TunnelStream {
    pub write: Box<dyn AsyncWrite + Send + Unpin>,
    pub read:  Box<dyn AsyncRead  + Send + Unpin>,
}
