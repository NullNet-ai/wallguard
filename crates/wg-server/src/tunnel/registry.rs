use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{oneshot, Mutex};

use crate::tunnel::TunnelStream;

#[derive(Clone)]
pub struct TunnelRegistry {
    inner: Arc<Mutex<HashMap<String, oneshot::Sender<TunnelStream>>>>,
}

impl TunnelRegistry {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(HashMap::new())) }
    }

    /// Register a tunnel_id before sending the open command to the agent.
    /// Returns a receiver that resolves when the agent's stream arrives.
    pub async fn register(&self, tunnel_id: &str) -> oneshot::Receiver<TunnelStream> {
        let (tx, rx) = oneshot::channel();
        self.inner.lock().await.insert(tunnel_id.to_string(), tx);
        rx
    }

    /// Claim a stream on behalf of an arriving agent connection.
    /// Returns `true` if a waiter was found and the stream dispatched.
    pub async fn claim(&self, tunnel_id: &str, stream: TunnelStream) -> bool {
        if let Some(tx) = self.inner.lock().await.remove(tunnel_id) {
            tx.send(stream).is_ok()
        } else {
            false
        }
    }
}
