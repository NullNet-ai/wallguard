use crate::context::Context;
use crate::control_channel::command::ExecutableCommand;
use crate::control_channel::commands::{
    HeartbeatCommand, OpenTtySessionCommand, OpenUiSessionCommand, UpdateTokenCommand,
};
use crate::storage::{Secret, Storage};
use crate::token_provider::TokenProvider;
use await_authorization::await_authorization;
use commands::OpenSshSessionCommand;
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use nullnet_libwallguard::{server_message, ClientMessage, ServerMessage};
use send_authenticate::send_authenticate;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tonic::Streaming;

mod await_authorization;
mod command;
mod commands;
mod send_authenticate;

pub(crate) type InboundStream = Arc<Mutex<Streaming<ServerMessage>>>;
pub(crate) type OutboundStream = Arc<Mutex<mpsc::Sender<ClientMessage>>>;

#[derive(Debug, Clone)]
pub struct ControlChannel {
    context: Context,
    uuid: String,
    org_id: String,
    token_provider: TokenProvider,
}

impl ControlChannel {
    pub fn new(context: Context, uuid: String, org_id: String) -> Self {
        Self {
            context,
            uuid,
            org_id,
            token_provider: TokenProvider::default(),
        }
    }

    pub async fn run(&self) -> Result<(), Error> {
        let (outbound, receiver) = mpsc::channel(64);
        let inbound = self
            .context
            .server
            .request_control_channel(receiver)
            .await?;

        let inbound = Arc::new(Mutex::new(inbound));
        let outbound = Arc::new(Mutex::new(outbound));

        match await_authorization(inbound.clone(), outbound.clone(), &self.uuid, &self.org_id)
            .await?
        {
            await_authorization::Verdict::Approved => {}
            await_authorization::Verdict::Rejected => {
                Err("Auhtorization has been rejected").handle_err(location!())?;
                // Cleanup ??
                // Remove ORG ID?
                // Enter some other state or something?
            }
        }

        send_authenticate(outbound).await?;

        while let Ok(message) = inbound.lock().await.message().await {
            let message = message
                .and_then(|message| message.message)
                .ok_or("Malformed message")
                .handle_err(location!())?;

            match message {
                server_message::Message::UpdateTokenCommand(token) => {
                    let cmd = UpdateTokenCommand::new(self.context.clone(), token);

                    if let Err(err) = cmd.execute().await {
                        log::error!("UpdateTokenCommand execution failed: {}", err.to_str());
                    }
                }
                server_message::Message::EnableNetworkMonitoringCommand(_) => todo!(),
                server_message::Message::EnableConfigurationMonitoringCommand(_) => todo!(),
                server_message::Message::EnableTelemetryMonitoringCommand(_) => todo!(),
                server_message::Message::OpenSshSessionCommand(ssh_session_data) => {
                    let cmd = OpenSshSessionCommand::new(self.context.clone(), ssh_session_data);

                    if let Err(err) = cmd.execute().await {
                        log::error!("OpenSshSessionCommand execution failed: {}", err.to_str());
                    }
                }
                server_message::Message::OpenTtySessionCommand(tunnel_token) => {
                    let cmd = OpenTtySessionCommand::new(self.context.clone(), tunnel_token);
                    if let Err(err) = cmd.execute().await {
                        log::error!("OpenTtySessionCommand execution failed: {}", err.to_str());
                    }
                }
                server_message::Message::OpenUiSessionCommand(ui_session_data) => {
                    let cmd = OpenUiSessionCommand::new(self.context.clone(), ui_session_data);

                    if let Err(err) = cmd.execute().await {
                        log::error!("OpenUiSessionCommand execution failed: {}", err.to_str());
                    }
                }
                server_message::Message::HeartbeatMessage(_) => {
                    let cmd = HeartbeatCommand::new();

                    if let Err(err) = cmd.execute().await {
                        log::error!("HeartbeatCommand execution failed: {}", err.to_str());
                    }
                }
                server_message::Message::DeviceDeauthorizedMessage(_) => {
                    // @TODO: Command
                    _ = Storage::delete_value(Secret::APP_ID).await;
                    _ = Storage::delete_value(Secret::APP_SECRET).await;
                    break;
                }
                server_message::Message::AuthorizationRejectedMessage(_) => {
                    Err("Unexpected message").handle_err(location!())?
                }

                server_message::Message::DeviceAuthorizedMessage(_) => {
                    Err("Unexpected message").handle_err(location!())?
                }
            }
        }

        log::info!("Control stream completed");

        Ok(())
    }
}
