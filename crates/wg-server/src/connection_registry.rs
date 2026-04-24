use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{broadcast, mpsc, RwLock};
use uuid::Uuid;

use crate::proto::control::ServerMessage;

pub type DeviceId = Uuid;

// ---------------------------------------------------------------------------
// Per-connection handle
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct DeviceConnection {
    pub org_id:       Uuid,
    pub out_tx:       mpsc::Sender<ServerMessage>,
    pub connected_at: Instant,
    /// Signalling this channel causes the connection task to shut down.
    pub shutdown_tx:  broadcast::Sender<()>,
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct ConnectionRegistry {
    inner: Arc<RwLock<HashMap<DeviceId, DeviceConnection>>>,
}

impl ConnectionRegistry {
    pub fn new() -> Self {
        Self { inner: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Insert a new connection. Signals any existing connection for this
    /// device to shut down before replacing it.
    pub async fn insert(&self, device_id: DeviceId, conn: DeviceConnection) {
        let mut map = self.inner.write().await;
        if let Some(old) = map.remove(&device_id) {
            tracing::info!(%device_id, "replacing stale connection");
            let _ = old.shutdown_tx.send(());
        }
        map.insert(device_id, conn);
    }

    /// Remove a connection (called when its task exits).
    pub async fn remove(&self, device_id: &DeviceId) {
        self.inner.write().await.remove(device_id);
    }

    /// Send a message to a specific device. Returns `false` if the device is
    /// not connected or its channel is closed.
    pub async fn send(&self, device_id: &DeviceId, msg: ServerMessage) -> bool {
        let map = self.inner.read().await;
        match map.get(device_id) {
            None       => false,
            Some(conn) => conn.out_tx.send(msg).await.is_ok(),
        }
    }

    /// Broadcast to every connected device.
    pub async fn broadcast(&self, msg: ServerMessage) {
        let map = self.inner.read().await;
        for conn in map.values() {
            let _ = conn.out_tx.send(msg.clone()).await;
        }
    }

    /// Returns `true` if no devices are currently connected.
    pub async fn is_empty(&self) -> bool {
        self.inner.read().await.is_empty()
    }

    /// Returns device IDs for all currently connected devices.
    pub async fn connected_device_ids(&self) -> Vec<DeviceId> {
        self.inner.read().await.keys().cloned().collect()
    }
}
