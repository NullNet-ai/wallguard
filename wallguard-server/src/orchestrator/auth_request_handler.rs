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
use crate::utilities;
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
            "Auth request received: code={}, uuid={}",
            auth.code,
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

        let root_token = match self.context.root_token_provider.get().await {
            Ok(token) => token,
            Err(err) => {
                log::error!("Failed to fetch root token: {}", err.to_str());
                let status = Status::internal("Internal Server Error: wrong root credentials");
                let _ = outbound.send(Err(status)).await;
                return;
            }
        };

        let sys_token = match self.context.sysdev_token_provider.get().await {
            Ok(token) => token,
            Err(err) => {
                log::error!("Failed to fetch sysdevice token: {}", err.to_str());
                let status =
                    Status::internal("Internal Server Error: wrong system device credentials");
                let _ = outbound.send(Err(status)).await;
                return;
            }
        };

        let installation_code = match self
            .context
            .datastore
            .obtain_installation_code(&auth.code, &root_token.jwt)
            .await
        {
            Ok(code) => {
                if code.is_none() {
                    let status = Status::internal(format!("Code {} is invalid", auth.code));
                    let _ = outbound.send(Err(status)).await;
                    return;
                }

                code.unwrap()
            }
            Err(err) => {
                log::error!("Failed to fetch installation code: {}", err.to_str());
                let status =
                    Status::internal("Internal Server Error: failed to fetch installation code");
                let _ = outbound.send(Err(status)).await;
                return;
            }
        };

        if !installation_code.redeemed {
            let mut device = match self
                .context
                .datastore
                .obtain_device_by_id(&root_token.jwt, &installation_code.device_id)
                .await
            {
                Ok(Some(device)) => device,
                Ok(None) => {
                    log::error!(
                        "Device assosiated with installtion code {} doest not exist",
                        auth.code
                    );

                    let status = Status::internal(format!(
                        "Device assosiated with installtion code {} doest not exist",
                        auth.code
                    ));
                    let _ = outbound.send(Err(status)).await;

                    return;
                }
                Err(_) => {
                    log::error!("Internal Server Error: failed to fetch device");

                    let status =
                        Status::internal(format!("Internal Server Error: failed to fetch device"));

                    let _ = outbound.send(Err(status)).await;
                    return;
                }
            };

            device.authorized = true;
            device.online = true;

            if self
                .context
                .datastore
                .update_device(&sys_token.jwt, &installation_code.device_id, &device)
                .await
                .is_err()
            {
                log::error!("Failed to update device");
                let status = Status::internal("Failed to update device");
                let _ = outbound.send(Err(status)).await;
                return;
            }

            if self
                .context
                .datastore
                .redeem_installation_code(&installation_code, &root_token.jwt)
                .await
                .is_err()
            {
                log::error!("Failed to redeem code");
                let status = Status::internal("Failed to redeem code");
                let _ = outbound.send(Err(status)).await;
                return;
            }

            let account_id = utilities::random::generate_random_string(12);
            let account_secret = utilities::random::generate_random_string(36);

            if self
                .context
                .datastore
                .register_device(&sys_token.jwt, &account_id, &account_secret, &device)
                .await
                .is_err()
            {
                log::error!("Failed to register device");
                let status = Status::internal("Failed to register device");
                let _ = outbound.send(Err(status)).await;
                return;
            }

            let client = Arc::new(Mutex::new(Client::new(
                auth.uuid.clone(),
                installation_code.organization_id,
                inbound,
                outbound,
                self.context.clone(),
            )));

            let mut authentication = AuthenticationData::default();

            authentication.app_id = Some(account_id);
            authentication.app_secret = Some(account_secret);

            if client.lock().await.authorize(authentication).await.is_ok() {
                clients.insert(auth.uuid, client.clone());
            } else {
                log::error!("Failed to authorize a device")
            }
        } else {
            let device = Device {
                authorized: false,
                uuid: auth.uuid.clone(),
                category: auth.category,
                model: auth.model,
                os: auth.target_os,
                online: true,
                organization: installation_code.organization_id.clone(),
                ..Default::default()
            };

            let client = Client::new(
                auth.uuid.clone(),
                installation_code.organization_id,
                inbound,
                outbound,
                self.context.clone(),
            );

            clients.insert(auth.uuid, Arc::new(Mutex::new(client)));
        }
    }
}
