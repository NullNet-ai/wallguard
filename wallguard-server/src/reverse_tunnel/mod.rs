use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::oneshot;
use tunnel_token::TokenHash;
use tunnel_token::TunnelToken;

mod tunnel_adapter;
mod tunnel_authentication_task;
mod tunnel_instance;
mod tunnel_token;

pub use tunnel_adapter::TunnelAdapter;
pub use tunnel_instance::TunnelInstance;

use crate::reverse_tunnel::tunnel_authentication_task::TunnelAuthenticationTask;

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

    /// Called when a new tunnel connection has been established.
    ///
    /// Spawns an asynchronous authentication task for the `TunnelInstance`. If
    /// authentication succeeds, the tunnel is forwarded to the registered
    /// listeners. Errors during authentication or forwarding will be handled
    /// by the task itself.
    pub fn on_new_tunnel_opened(&self, instance: TunnelInstance) {
        let task = TunnelAuthenticationTask::new(instance, self.listeners.clone());
        tokio::spawn(task.authenticate());
    }
}
