use client::Client;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tonic::Status;

use crate::orchestrator::authorization_stream::{AuthorizationStream, PendingAuth};
use crate::protocol::wallguard_commands::WallGuardCommand;
use crate::token_provider::TokenProvider;

mod authorization_stream;
mod client;
mod control_stream;

type ClientsMap = Arc<Mutex<HashMap<String, Client>>>;
type PendingAuthMap = Arc<Mutex<HashMap<String, PendingAuth>>>;

#[derive(Debug, Clone, Default)]
pub struct Orchestrator {
    clients: ClientsMap,
    pending_auths: PendingAuthMap,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn is_client_connected(&self, device_uuid: &str) -> bool {
        self.clients.lock().await.contains_key(device_uuid)
    }

    pub async fn on_client_connected(
        &self,
        device_uuid: &str,
        token_provider: TokenProvider,
        control_stream: mpsc::Sender<Result<WallGuardCommand, Status>>,
    ) -> Result<(), Error> {
        log::debug!("Client connected UUID {}", device_uuid);

        if self.is_client_connected(device_uuid).await {
            let message = format!("Client with UUID '{device_uuid}' is already connected");
            return Err(message).handle_err(location!())?;
        }

        let client = Client::new(device_uuid, token_provider, control_stream, self.clone());

        self.clients.lock().await.insert(device_uuid.into(), client);

        Ok(())
    }

    pub async fn on_client_disconnected(&self, device_uuid: &str) -> Result<(), Error> {
        log::debug!("Client disconnected UUID {}", device_uuid);

        if self.clients.lock().await.remove(device_uuid).is_none() {
            Err(format!(
                "Device with UUID '{}' is not connected",
                device_uuid
            ))
            .handle_err(location!())?;
        }
        Ok(())
    }

    pub async fn is_auth_pending(&self, device_uuid: &str) -> bool {
        self.pending_auths.lock().await.contains_key(device_uuid)
    }

    pub async fn on_client_requested_authorization(
        &self,
        device_uuid: &str,
        stream: AuthorizationStream,
    ) -> Result<(), Error> {
        log::debug!("Client requested authorization UUID {}", device_uuid);

        if self.is_auth_pending(device_uuid).await {
            return Err(format!(
                "Authorization already pending for device UUID {}",
                device_uuid
            ))
            .handle_err(location!());
        }

        let auth = PendingAuth::new(device_uuid, stream, self.clone());

        self.pending_auths
            .lock()
            .await
            .insert(device_uuid.into(), auth);

        Ok(())
    }

    pub async fn on_client_authorization_completed(&self, device_uuid: &str) -> Result<(), Error> {
        log::debug!("Client authorization completed UUID {}", device_uuid);

        if self
            .pending_auths
            .lock()
            .await
            .remove(device_uuid)
            .is_none()
        {
            return Err(format!(
                "No pending authorization found for device UUID {}",
                device_uuid
            ))
            .handle_err(location!());
        }

        Ok(())
    }

    pub async fn get_client(&self, device_uuid: &str) -> Option<Client> {
        self.clients.lock().await.get(device_uuid).cloned()
    }

    pub async fn get_pending_auth(&self, device_uuid: &str) -> Option<PendingAuth> {
        self.pending_auths.lock().await.get(device_uuid).cloned()
    }
}
