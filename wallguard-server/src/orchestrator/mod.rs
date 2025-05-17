use client::Client;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tonic::Status;

use crate::protocol::wallguard_commands::WallGuardCommand;
use crate::token_provider::TokenProvider;

mod client;
mod stream;

type ClientsMap = Arc<Mutex<HashMap<String, Client>>>;

#[derive(Debug, Clone)]
pub struct Orchestrator {
    clients: ClientsMap,
    complete_tx: mpsc::Sender<String>,
}

impl Orchestrator {
    pub fn new() -> Self {
        let (complete_tx, complete_rx) = mpsc::channel(64);
        let clients = Arc::new(Mutex::new(HashMap::new()));

        tokio::spawn(clients_lifecycle_monitor(clients.clone(), complete_rx));

        Self {
            clients,
            complete_tx,
        }
    }

    pub async fn is_client_connected(&self, device_id: &str) -> bool {
        self.clients.lock().await.contains_key(device_id)
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

        let client = Client::new(
            device_id,
            token_provider,
            control_stream,
            self.complete_tx.clone(),
        );

        self.clients.lock().await.insert(device_id.into(), client);

        Ok(())
    }

    pub async fn get_client(&self, device_id: &str) -> Option<Client> {
        self.clients.lock().await.get(device_id).cloned()
    }
}

/// Monitors the lifecycle of connected clients by listening for completion signals.
///
/// This task runs in the background, waiting for client `device_id`s to be sent through
/// the `receiver` channel. When a message is received, it removes the corresponding client
/// from the shared `clients` map. If the client is not found, an error is logged.
///
/// This monitor is expected to run indefinitely. If the `receiver` channel is closed and
/// the loop exits, the function panics, as this is considered an unrecoverable state.
///
/// # Parameters
/// - `clients`: A shared, mutex-protected map that tracks active clients by device ID.
/// - `receiver`: A channel that receives `device_id`s of clients whose control stream tasks have completed.
async fn clients_lifecycle_monitor(clients: ClientsMap, mut receiver: mpsc::Receiver<String>) {
    while let Some(device_id) = receiver.recv().await {
        if clients.lock().await.remove(&device_id).is_none() {
            log::error!(
                "Orchestrator received a client completion event, but found no record of a client with device ID '{}'",
                device_id
            );
        }
    }

    panic!("Orchestrator's lifecycle monitor terminated unexpectedly. This should never happen.");
}
