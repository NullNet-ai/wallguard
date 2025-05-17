pub use config::ReverseTunnelConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::sync::oneshot;
use tunnel_token::TokenHash;
use tunnel_token::TunnelToken;

mod config;
mod tunnel_token;

type ListenersMap = Arc<Mutex<HashMap<TokenHash, oneshot::Sender<TcpStream>>>>;

#[derive(Debug, Clone)]
pub struct ReverseTunnel {
    listeners: ListenersMap,
}

impl ReverseTunnel {
    /// Creates a new reverse tunnel and starts the background task.
    pub fn new() -> Self {
        let config = ReverseTunnelConfig::from_env();
        let listeners = Arc::new(Mutex::new(HashMap::new()));

        tokio::spawn(tunnel_task(config, listeners.clone()));

        Self { listeners }
    }

    /// Generates a new tunnel token and prepares to receive a connection identified by its hash.
    ///
    /// Returns the raw token (to be used by the remote client) and a `Receiver`
    /// that resolves when a client connects using the matching token hash.
    pub async fn expect_connection(&self) -> (TunnelToken, oneshot::Receiver<TcpStream>) {
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

/// The main reverse tunnel listener task.
///
/// This function binds a TCP listener to the configured address and continuously accepts
/// incoming connections. Each new connection is expected to send a 32-byte SHA-256 hash
/// representing the authentication token. If a matching listener is registered with this
/// hash, the TCP stream is forwarded to it via a `oneshot::Sender`.
///
/// # Parameters
/// - `config`: The reverse tunnel configuration containing the address to bind.
/// - `listeners`: A shared map of expected connection hashes to their corresponding receivers.
async fn tunnel_task(config: ReverseTunnelConfig, listeners: ListenersMap) {
    log::info!("Reverse tunnel listening on {}", config.addr);

    let listener = match TcpListener::bind(config.addr).await {
        Ok(listener) => listener,
        Err(err) => {
            log::error!(
                "Failed to bind reverse tunnel to address {}: {}",
                config.addr,
                err
            );
            std::process::exit(1);
        }
    };

    loop {
        let (mut stream, _) = match listener.accept().await {
            Ok((stream, addr)) => {
                log::debug!("Incoming connection from {}", addr);
                (stream, addr)
            }
            Err(err) => {
                log::error!("Reverse tunnel failed to accept connection: {}", err);
                continue;
            }
        };

        let listeners = listeners.clone();
        tokio::spawn(async move {
            let hash = match TokenHash::read_from_stream(&mut stream).await {
                Ok(hash) => hash,
                Err(err) => {
                    log::error!(
                        "Failed to read token hash from tunnel stream: {}",
                        err.to_str()
                    );
                    return shutdown_stream(stream).await;
                }
            };

            match listeners.lock().await.remove(&hash) {
                Some(channel) => {
                    if let Err(stream) = channel.send(stream) {
                        log::error!(
                            "Reverse tunnel failed to forward TCP stream: receiver dropped"
                        );
                        return shutdown_stream(stream).await;
                    }
                }
                None => {
                    log::warn!(
                        "Received tunnel connection with unknown token hash: {:?}",
                        hash
                    );
                    return shutdown_stream(stream).await;
                }
            }
        });
    }
}

async fn shutdown_stream(mut stream: TcpStream) {
    use tokio::io::AsyncWriteExt;

    if let Err(e) = stream.shutdown().await {
        log::warn!("Failed to shutdown stream: {}", e);
    }
}
