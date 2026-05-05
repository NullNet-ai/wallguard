pub mod http;
pub mod remote_desktop;
pub mod ssh;
pub mod transport;
pub mod tty;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt as _};
use tokio::sync::Mutex;

use crate::config::Config;

pub const HELLO_LEN: usize = 36;

pub struct TunnelStream {
    pub write: Box<dyn AsyncWrite + Send + Unpin>,
    pub read:  Box<dyn AsyncRead  + Send + Unpin>,
}

/// Per-session shared state for all tunnel operations.
pub struct TunnelContext {
    pub config:      Arc<Config>,
    /// Rustls client config built once per agent run and reused across all
    /// tunnel connections — avoids re-reading cert/key PEM files on every
    /// tunnel command.
    pub tls:         Arc<rustls::ClientConfig>,
    /// Lazily-initialised QUIC connection; None until first tunnel command.
    pub quic_conn:   Arc<Mutex<Option<quinn::Connection>>>,
    /// Set true after any persistent QUIC failure; skips QUIC for remainder
    /// of the gRPC session and goes straight to TCP-TLS.
    pub quic_failed: Arc<AtomicBool>,
}

impl TunnelContext {
    pub fn new(config: Arc<Config>, tls: Arc<rustls::ClientConfig>) -> Self {
        Self {
            config,
            tls,
            quic_conn:   Arc::new(Mutex::new(None)),
            quic_failed: Arc::new(AtomicBool::new(false)),
        }
    }
}

/// Write the 36-byte tunnel hello (UUID string) before any relay data.
pub async fn write_hello(
    stream:    &mut (impl AsyncWrite + Unpin),
    tunnel_id: &str,
) -> anyhow::Result<()> {
    if tunnel_id.len() != HELLO_LEN {
        anyhow::bail!(
            "tunnel_id must be {} bytes (UUID format), got {}",
            HELLO_LEN,
            tunnel_id.len(),
        );
    }
    stream.write_all(tunnel_id.as_bytes()).await?;
    Ok(())
}

/// Bidirectional relay: copies r1→w2 and r2→w1 concurrently.
/// Returns as soon as either direction reaches EOF or error.
pub async fn relay(
    r1: Box<dyn AsyncRead  + Send + Unpin>,
    w1: Box<dyn AsyncWrite + Send + Unpin>,
    r2: Box<dyn AsyncRead  + Send + Unpin>,
    w2: Box<dyn AsyncWrite + Send + Unpin>,
) {
    let h1 = tokio::spawn(async move {
        let (mut r, mut w) = (r1, w2);
        tokio::io::copy(&mut r, &mut w).await
    });
    let h2 = tokio::spawn(async move {
        let (mut r, mut w) = (r2, w1);
        tokio::io::copy(&mut r, &mut w).await
    });
    tokio::select! {
        _ = h1 => {}
        _ = h2 => {}
    }
}
