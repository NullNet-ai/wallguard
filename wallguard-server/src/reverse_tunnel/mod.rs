use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

use tokio::sync::Mutex;
use tokio::sync::oneshot;
use tunnel_token::TunnelToken;
use tunnel_token::{HandshakeFailure, TokenHash};

mod config;
mod tunnel_instance;
mod tunnel_token;

pub use tunnel_instance::TunnelInstance;

use crate::app_context::AppContext;

/// How long a freshly accepted connection may take to send its token hash.
/// Without a bound, idle connections (scanners, half-open agents) pile up and
/// eventually exhaust the process's file descriptors.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

/// Pause after a failed `accept()`. Errors such as EMFILE are persistent, so
/// retrying immediately turns the accept loop into a CPU-bound spin.
const ACCEPT_ERROR_BACKOFF: Duration = Duration::from_millis(100);

pub type ListenersMap = Arc<Mutex<HashMap<TokenHash, oneshot::Sender<TunnelInstance>>>>;

#[derive(Debug, Clone)]
pub struct ReverseTunnel {
    listeners: ListenersMap,
}

impl ReverseTunnel {
    /// Creates a new reverse tunnel.
    pub fn new() -> Self {
        let listeners = Arc::new(Mutex::new(HashMap::new()));

        Self { listeners }
    }

    /// Generates a new tunnel token and prepares to receive a connection identified by its hash.
    ///
    /// Returns the raw token (to be used by the remote client) and a `Receiver`
    /// that resolves when a client connects using the matching token hash.
    pub async fn expect_connection(&self) -> (TunnelToken, oneshot::Receiver<TunnelInstance>) {
        let token = TunnelToken::generate();

        let (tx, rx) = oneshot::channel();

        self.listeners.lock().await.insert(token.clone().into(), tx);

        (token, rx)
    }

    /// Cancels an expected connection associated with the given token.
    ///
    /// If the token hash was present, it is removed and the corresponding sender is dropped.
    /// Returns `true` if an entry was removed, `false` if it wasn't found.
    pub async fn cancel_expectation(&self, token: &TunnelToken) -> bool {
        let hash: TokenHash = token.clone().into();
        self.listeners.lock().await.remove(&hash).is_some()
    }
}

pub async fn run_tunnel_acceptor(context: AppContext) -> Result<(), Error> {
    let config = config::Config::from_env();

    let listener = tokio::net::TcpListener::bind(config.addr)
        .await
        .handle_err(location!())?;

    loop {
        let (mut stream, peer) = match listener.accept().await {
            Ok(accepted) => accepted,
            Err(err) => {
                log::error!("Tunnel acceptor: failed to accept connection: {err}");
                tokio::time::sleep(ACCEPT_ERROR_BACKOFF).await;
                continue;
            }
        };

        let ctx = context.clone();

        tokio::spawn(async move {
            /*
             * TODO
             * Send Confirmation or Rejection message to the client
             */

            log::debug!("Tunnel acceptor: accepted connection from {peer}");

            let hash = match TokenHash::read_from_stream(&mut stream, HANDSHAKE_TIMEOUT).await {
                Ok(hash) => hash,
                Err(err) => {
                    // A connection that closes without sending anything is
                    // usually a port probe or TCP health check, not an agent.
                    if matches!(err.failure, HandshakeFailure::Closed) && err.received.is_empty() {
                        log::info!(
                            "Tunnel handshake from {peer}: connection closed without sending data"
                        );
                    } else {
                        log::error!("Tunnel handshake from {peer} failed: {err}");
                    }
                    let _ = stream.shutdown().await;
                    return;
                }
            };

            let mut tunnel = TunnelInstance::from(stream);

            match ctx.tunnel.listeners.lock().await.remove(&hash) {
                Some(channel) => {
                    if let Err(mut tunnel) = channel.send(tunnel) {
                        let _ = tunnel.shutdown().await;
                        log::error!(
                            "Failed to hand over tunnel from {peer}: the requester is no longer waiting"
                        );
                    } else {
                        log::debug!("Tunnel acceptor: tunnel from {peer} established");
                    }
                }
                None => {
                    log::warn!(
                        "Received tunnel connection from {peer} with unknown token hash: {:?}",
                        hash
                    );

                    let _ = tunnel.shutdown().await;
                }
            };
        });
    }
}
