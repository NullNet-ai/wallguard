use crate::context::Context;
use crate::control_channel::command::ExecutableCommand;
use crate::control_channel::commands::{
    HeartbeatCommand, OpenTtySessionCommand, OpenUiSessionCommand, UpdateTokenCommand,
};
use crate::daemon::Daemon;
use crate::storage::{Secret, Storage};
use await_authorization::await_authorization;
use commands::OpenSshSessionCommand;
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use nullnet_libwallguard::{server_message, ClientMessage, ServerMessage};
use send_authenticate::send_authenticate;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Mutex};
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
    terminate: broadcast::Sender<()>,
}

impl ControlChannel {
    pub fn new(context: Context, uuid: String, org_id: String) -> Self {
        let (terminate, _) = broadcast::channel(1);

        tokio::spawn(stream_wrapper(
            context.clone(),
            uuid.clone(),
            org_id.clone(),
            terminate.subscribe(),
        ));

        Self {
            context,
            uuid,
            org_id,
            terminate,
        }
    }

    pub fn get_uuid(&self) -> String {
        self.uuid.clone()
    }

    pub fn get_org_id(&self) -> String {
        self.org_id.clone()
    }

    pub fn terminate(&self) {
        let _ = self.terminate.send(());
    }
}

async fn stream_wrapper(
    context: Context,
    uuid: String,
    org_id: String,
    mut terminate: broadcast::Receiver<()>,
) {
    tokio::select! {
        _ = terminate.recv() => {}
        result = control_stream(context.clone(), &uuid, &org_id) => {
            if let Err(err) = result {
                Daemon::on_error(context.daemon, err.to_str()).await;
            }
        }
    };
}

async fn control_stream(context: Context, uuid: &str, org_id: &str) -> Result<(), Error> {
    let (outbound, receiver) = mpsc::channel(64);
    let inbound = context.server.request_control_channel(receiver).await?;

    let inbound = Arc::new(Mutex::new(inbound));
    let outbound = Arc::new(Mutex::new(outbound));

    match await_authorization(inbound.clone(), outbound.clone(), uuid, org_id).await? {
        await_authorization::Verdict::Approved => {}
        await_authorization::Verdict::Rejected => {
            Err("Auhtorization has been rejected").handle_err(location!())?;
            // Cleanup ??
            // Remove ORG ID?
            // Enter some other state or something?
        }
    }

    // Clone the outbound stream to keep it alive—closing it signals
    // an error to the server, which closes the connection.
    send_authenticate(outbound.clone()).await?;

    while let Ok(message) = inbound.lock().await.message().await {
        let message = message
            .and_then(|message| message.message)
            .ok_or("Malformed message")
            .handle_err(location!())?;

        match message {
            server_message::Message::UpdateTokenCommand(token) => {
                let cmd = UpdateTokenCommand::new(context.clone(), token);

                if let Err(err) = cmd.execute().await {
                    log::error!("UpdateTokenCommand execution failed: {}", err.to_str());
                }
            }
            server_message::Message::EnableNetworkMonitoringCommand(_) => todo!(),
            server_message::Message::EnableConfigurationMonitoringCommand(_) => todo!(),
            server_message::Message::EnableTelemetryMonitoringCommand(_) => todo!(),
            server_message::Message::OpenSshSessionCommand(ssh_session_data) => {
                let cmd = OpenSshSessionCommand::new(context.clone(), ssh_session_data);

                if let Err(err) = cmd.execute().await {
                    log::error!("OpenSshSessionCommand execution failed: {}", err.to_str());
                }
            }
            server_message::Message::OpenTtySessionCommand(tunnel_token) => {
                let cmd = OpenTtySessionCommand::new(context.clone(), tunnel_token);
                if let Err(err) = cmd.execute().await {
                    log::error!("OpenTtySessionCommand execution failed: {}", err.to_str());
                }
            }
            server_message::Message::OpenUiSessionCommand(ui_session_data) => {
                let cmd = OpenUiSessionCommand::new(context.clone(), ui_session_data);

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
                // Gracefuly transition to IDLE state
                todo!();
            }
            server_message::Message::AuthorizationRejectedMessage(_) => {
                Err("Unexpected message").handle_err(location!())?
            }

            server_message::Message::DeviceAuthorizedMessage(_) => {
                Err("Unexpected message").handle_err(location!())?
            }
        }
    }

    Ok(())
}
