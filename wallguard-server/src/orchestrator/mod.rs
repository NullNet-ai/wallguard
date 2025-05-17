use client::Client;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tonic::Status;

use crate::datastore::Datastore;
use crate::protocol::wallguard_commands::WallGuardCommand;
use crate::token_provider::TokenProvider;

mod client;

#[derive(Debug)]
struct OrchestratorInner {
    pub(crate) clients: HashMap<String, Client>,
}

impl OrchestratorInner {
    pub fn new() -> Self {
        Self {
            clients: HashMap::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Orchestrator {
    inner: Arc<Mutex<OrchestratorInner>>,
    datastore: Datastore,
}

impl Orchestrator {
    pub fn new(datastore: Datastore) -> Self {
        let inner = Arc::new(Mutex::new(OrchestratorInner::new()));
        Self { inner, datastore }
    }

    pub async fn is_client_connected(&self, device_id: &str) -> bool {
        self.inner.lock().await.clients.contains_key(device_id)
    }

    pub async fn on_client_connected(
        &self,
        device_id: &str,
        token_provider: TokenProvider,
        control_stream: mpsc::Sender<Result<WallGuardCommand, Status>>,
    ) -> Result<(), Error> {
        if self.is_client_connected(device_id).await {
            let message = format!("Client with id '{device_id}' is already connected");
            log::error!("{message}");
            return Err(message).handle_err(location!())?;
        }

        let client = Client::new(device_id, token_provider, control_stream);

        self.inner
            .lock()
            .await
            .clients
            .insert(device_id.into(), client);

        Ok(())
    }

    pub async fn get_client(&self, device_id: &str) -> Option<Client> {
        self.inner.lock().await.clients.get(device_id).cloned()
    }
}
