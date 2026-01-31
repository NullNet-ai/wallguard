use nullnet_liberror::{Error, ErrorHandler, Location, location};
use tokio::io::AsyncWriteExt;
use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::oneshot;
use tunnel_token::TokenHash;
use tunnel_token::TunnelToken;

mod config;
mod tunnel_instance;
mod tunnel_token;

pub use tunnel_instance::TunnelInstance;

use crate::app_context::AppContext;

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
        let Ok((mut stream, _)) = listener.accept().await else {
            continue;
        };

        let ctx = context.clone();

        tokio::spawn(async move {
            /*
             * TODO
             * Send Confirmation or Rejection message to the client
             */

            let Ok(hash) = TokenHash::read_from_stream(&mut stream).await else {
                log::error!("Faile to read token hash from newely accepted TCP stream");
                let _ = stream.shutdown().await;
                return;
            };

            let mut tunnel = TunnelInstance::from(stream);

            match ctx.tunnel.listeners.lock().await.remove(&hash) {
                Some(channel) => {
                    if let Err(mut tunnel) = channel.send(tunnel) {
                        let _ = tunnel.shutdown().await;
                        log::error!("Failed to send tunnel instance");
                    }
                }
                None => {
                    log::warn!(
                        "Received tunnel connection with unknown token hash: {:?}",
                        hash
                    );

                    let _ = tunnel.shutdown().await;
                }
            };
        });
    }
}
