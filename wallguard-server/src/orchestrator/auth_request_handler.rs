//! Handles the initial authorization of new client connections.
//!
//! This component is responsible for processing incoming `AuthorizationRequest`s,
//! validating and registering clients, and syncing device records with the datastore.
//! It prevents duplicate connections and ensures proper authorization flow based on device status.
//!
//! If the device does not exist in the datastore, it will be created in an unauthorized state.
//! If the device is already authorized, the handler attempts to authorize the client session.
//!
//! For rejected or failed authorization attempts, appropriate error messages are sent back via the outbound stream.

use crate::app_context::AppContext;
use crate::datastore::Device;
use crate::orchestrator::client::{Client, InboundStream, OutboundStream};
use crate::protocol::wallguard_commands::server_message::Message;
use crate::protocol::wallguard_commands::{
    AuthenticationData, AuthorizationRequest, ServerMessage,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::Status;

pub struct AuthReqHandler {
    context: AppContext,
}

impl AuthReqHandler {
    pub fn new(context: AppContext) -> Self {
        Self { context }
    }

    pub async fn handle(
        self,
        inbound: InboundStream,
        outbound: OutboundStream,
        auth: AuthorizationRequest,
    ) {
        log::info!(
            "Auth request received: org_id={}, uuid={}",
            auth.org_id,
            auth.uuid
        );

        let mut clients = self.context.orchestractor.clients.lock().await;

        if clients.contains_key(&auth.uuid) {
            log::warn!(
                "Rejecting duplicate connection: device UUID {} is already connected",
                auth.uuid
            );
            let _ = outbound
                .send(Ok(ServerMessage {
                    message: Some(Message::AuthorizationRejectedMessage(())),
                }))
                .await;
            return;
        }

        let token = match self
            .context
            .token_provider
            .get()
            .await
            .map_err(|err| Status::internal(err.to_str()))
        {
            Ok(token) => token,
            Err(status) => {
                log::error!("Failed to fetch system token: {}", status);
                let _ = outbound.send(Err(status)).await;
                return;
            }
        };

        let device = match self
            .context
            .datastore
            .obtain_device_by_uuid(&token.jwt, &auth.uuid)
            .await
            .map_err(|err| Status::internal(err.to_str()))
        {
            Ok(device) => device,
            Err(status) => {
                log::error!("Failed to fetch device: {}", status);
                let _ = outbound.send(Err(status)).await;
                return;
            }
        };

        if device.is_none() {
            let mut device = Device::default();
            device.authorized = false;
            device.uuid = auth.uuid.clone();

            if let Err(status) = self
                .context
                .datastore
                .create_device(&token.jwt, &device, Some(auth.org_id.clone()))
                .await
                .map_err(|err| Status::internal(err.to_str()))
            {
                log::error!("Failed to create device: {}", status);
                let _ = outbound.send(Err(status)).await;
                return;
            }

            let client = Client::new(
                auth.uuid.clone(),
                auth.org_id,
                inbound,
                outbound,
                self.context.clone(),
            );

            clients.insert(auth.uuid, Arc::new(Mutex::new(client)));
        } else {
            let device = device.unwrap();

            if !device.authorized {
                let client = Client::new(
                    auth.uuid.clone(),
                    auth.org_id,
                    inbound,
                    outbound,
                    self.context.clone(),
                );

                self.context
                    .orchestractor
                    .clients
                    .lock()
                    .await
                    .insert(auth.uuid, Arc::new(Mutex::new(client)));
            } else {
                let client = Client::new(
                    auth.uuid.clone(),
                    auth.org_id,
                    inbound,
                    outbound,
                    self.context.clone(),
                );

                let client = Arc::new(Mutex::new(client));

                if client
                    .lock()
                    .await
                    .authorize(AuthenticationData::default())
                    .await
                    .is_ok()
                {
                    clients.insert(auth.uuid, client.clone());
                } else {
                    log::error!("Failed to authorize a device")
                }
            }
        }
    }
}
