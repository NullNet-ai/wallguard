use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::time::Duration;
use tokio::sync::broadcast;

use crate::app_context::AppContext;
use crate::datastore::HeartbeatModel;
use crate::orchestrator::client::{InboundStream, OutboundStream};
use crate::token_provider::TokenProvider;
use wallguard_common::protobuf::wallguard_commands::{
    ExecuteCliCommandResponse, ServerMessage, client_message, server_message,
};

const TOKEN_UPDATE_TIME: Duration = Duration::from_secs(60);

pub(crate) async fn control_stream(
    device_id: String,
    instance_id: String,
    inbound: InboundStream,
    outbound: OutboundStream,
    context: AppContext,
    channel: broadcast::Sender<ExecuteCliCommandResponse>,
) {
    log::info!("Starting a control stream for device ID {device_id}, Instance {instance_id}");

    if let Ok(token) = context.sysdev_token_provider.get().await {
        if context
            .datastore
            .update_device_online_status(&token.jwt, &device_id, true)
            .await
            .is_err()
        {
            log::error!("Failed to update device record");
        }
    } else {
        log::error!("Failed to obtain system device token");
    }

    if let Err(err) = authstream(
        inbound,
        outbound,
        context.clone(),
        channel,
        device_id.clone(),
    )
    .await
    {
        log::error!(
            "Control stream for client with device ID '{}' failed: {}",
            device_id,
            err.to_str()
        );
    }

    let _ = context
        .orchestractor
        .on_disconnected(&device_id, &instance_id)
        .await;

    if let Ok(token) = context.sysdev_token_provider.get().await {
        let _ = context
            .datastore
            .delete_device_instance(&token.jwt, &instance_id)
            .await;

        let is_online = context
            .orchestractor
            .does_client_have_connected_instances(&device_id)
            .await;

        if context
            .datastore
            .update_device_online_status(&token.jwt, &device_id, is_online)
            .await
            .is_err()
        {
            log::error!("Failed to update device record");
        }
    } else {
        log::error!("Failed to obtain system device token");
    }
}

async fn authstream(
    mut inbound: InboundStream,
    outbound: OutboundStream,
    context: AppContext,
    channel: broadcast::Sender<ExecuteCliCommandResponse>,
    device_id: String,
) -> Result<(), Error> {
    let message = inbound
        .message()
        .await
        .handle_err(location!())?
        .ok_or("Client sent an empty message")
        .handle_err(location!())?
        .message
        .ok_or("Malformed message (missing payload)")
        .handle_err(location!())?;

    let authentication = match message {
        client_message::Message::Authentication(authentication) => authentication,
        _ => Err("Unexpected message").handle_err(location!())?,
    };

    let token_provider = TokenProvider::new(
        authentication.app_id,
        authentication.app_secret,
        false,
        context.datastore.clone(),
    );

    let mut token_update_interval = tokio::time::interval(TOKEN_UPDATE_TIME);

    loop {
        tokio::select! {
            _ = token_update_interval.tick() => {
                outbound
                    .send(Ok(ServerMessage {
                        message: Some(server_message::Message::UpdateTokenCommand(
                            token_provider.get().await?.jwt.clone(),
                        )),
                    }))
                    .await
                    .handle_err(location!())?;
            }

            msg = inbound.message() => {
                match msg {
                    Ok(Some(message)) => {
                        let Some(msg) = message.message else {
                            log::warn!("Received message wrapper but no inner `message`; ignoring");
                            continue;
                        };

                        match msg {
                            client_message::Message::ExecuteCliCommandResponse(response) => {
                                if let Err(err) = channel.send(response) {
                                    log::error!("Failed to send response to the channel: {err}");
                                }
                            },
                            client_message::Message::Heartbeat(()) => {
                                log::debug!("Received a heartbeat from {device_id}");
                                if let Ok(token) = context.sysdev_token_provider.get().await {
                                    let data = HeartbeatModel::from_device_id(device_id.clone());
                                    if context
                                        .datastore
                                        .create_heartbeat(&token.jwt, &data)
                                        .await
                                        .is_err()
                                    {
                                        log::error!("Failed to write heatbeat");
                                    }
                                } else {
                                    log::error!("Heartbeat: Failed to obtain token");
                                }
                            }
                            other => {
                                log::warn!("Unexpected message from client after authentication; ignoring: {:?}", other);
                            },
                        }
                    }
                    Ok(None) => {
                        return Err("Inbound stream closed by client").handle_err(location!());
                    }
                    Err(e) => {
                        return Err(format!("Inbound stream error: {e}")).handle_err(location!());
                    }
                }
            }
        }
    }
}
