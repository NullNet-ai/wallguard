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
use crate::datastore::{Device, DeviceCredentials};
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

        let root_token = match self
            .context
            .root_token_provider
            .get()
            .await
            .map_err(|err| Status::internal(err.to_str()))
        {
            Ok(token) => token,
            Err(status) => {
                log::error!("Failed to fetch root token: {}", status);
                let _ = outbound.send(Err(status)).await;
                return;
            }
        };

        let sys_token = match self
            .context
            .sysdev_token_provider
            .get()
            .await
            .map_err(|err| Status::internal(err.to_str()))
        {
            Ok(token) => token,
            Err(status) => {
                log::error!("Failed to fetch sysdevice token: {}", status);
                let _ = outbound.send(Err(status)).await;
                return;
            }
        };

        let device = match self
            .context
            .datastore
            .obtain_device_by_uuid(&root_token.jwt, &auth.uuid)
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
            // If there is no device with such UUID, we do
            // - create new device & account
            // - save credential into the temporary table

            let credentials = DeviceCredentials::generate(auth.uuid.clone());

            let response = match self
                .context
                .datastore
                .register_device(
                    &sys_token.jwt,
                    &credentials.account_id,
                    &credentials.account_secret,
                    &auth.org_id,
                )
                .await
                .map_err(|err| Status::internal(err.to_str()))
            {
                Ok(response) => response,
                Err(status) => {
                    log::error!("Failed to create device: {}", status);
                    let _ = outbound.send(Err(status)).await;
                    return;
                }
            };

            let device = Device {
                authorized: false,
                uuid: auth.uuid.clone(),
                category: auth.category,
                model: auth.model,
                os: auth.target_os,
                online: true,
                ..Default::default()
            };

            if let Err(status) = self
                .context
                .datastore
                .update_device(&sys_token.jwt, &response.device_id, &device)
                .await
                .map_err(|err| Status::internal(err.to_str()))
            {
                log::error!("Failed to update device: {}", status);
                let _ = outbound.send(Err(status)).await;
                return;
            };

            if let Err(status) = self
                .context
                .datastore
                .create_device_credentials(&sys_token.jwt, &credentials)
                .await
                .map_err(|err| Status::internal(err.to_str()))
            {
                log::error!("Failed to create device credentials: {}", status);
                let _ = outbound.send(Err(status)).await;
                return;
            };

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

            // @TODO:
            // Here we can check if device's data (`model`, `target_os` or `category`) has changed
            // and act accordingly.

            if let Err(status) = self
                .context
                .datastore
                .update_device_online_status(&sys_token.jwt, &device.uuid, true)
                .await
                .map_err(|err| Status::internal(err.to_str()))
            {
                log::error!("Failed to udpate device record: {}", status);
                let _ = outbound.send(Err(status)).await;
                return;
            }

            if !device.authorized {
                let client = Client::new(
                    auth.uuid.clone(),
                    auth.org_id,
                    inbound,
                    outbound,
                    self.context.clone(),
                );

                clients.insert(auth.uuid, Arc::new(Mutex::new(client)));
            } else {
                let credentials = match self
                    .context
                    .datastore
                    .obtain_device_credentials(&sys_token.jwt, &auth.uuid)
                    .await
                    .map_err(|err| Status::internal(err.to_str()))
                {
                    Ok(credentials) => credentials,
                    Err(status) => {
                        log::error!("Failed to obtain device credentials: {}", status);
                        let _ = outbound.send(Err(status)).await;
                        return;
                    }
                };

                let mut authentication = AuthenticationData::default();

                if let Some(credentials) = credentials {
                    authentication.app_id = Some(credentials.account_id);
                    authentication.app_secret = Some(credentials.account_secret);

                    if let Err(status) = self
                        .context
                        .datastore
                        .delete_device_credentials(&sys_token.jwt, &credentials.id)
                        .await
                        .map_err(|err| Status::internal(err.to_str()))
                    {
                        log::error!("Failed to redeem device credentials");
                        let _ = outbound.send(Err(status)).await;
                        return;
                    }
                }

                let client = Arc::new(Mutex::new(Client::new(
                    auth.uuid.clone(),
                    auth.org_id,
                    inbound,
                    outbound,
                    self.context.clone(),
                )));

                if client.lock().await.authorize(authentication).await.is_ok() {
                    clients.insert(auth.uuid, client.clone());
                } else {
                    log::error!("Failed to authorize a device")
                }
            }
        }
    }
}