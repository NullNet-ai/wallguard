use crate::context::Context;
use crate::control_channel::command::ExecutableCommand;
use crate::control_channel::commands::{
    CreateAliasCommand, CreateFilterRuleCommand, CreateNatRuleCommand,
    EnableConfigurationMonitoringCommand, EnableNetworkMonitoringCommand,
    EnableTelemetryMonitoringCommand, HeartbeatCommand, OpenRemoteDesktopSessionCommand,
    OpenTtySessionCommand, OpenUiSessionCommand, UpdateTokenCommand,
};
use crate::control_channel::post_startup::post_startup;
use crate::daemon::Daemon;
use crate::storage::{Secret, Storage};
use await_authorization::await_authorization;
use commands::OpenSshSessionCommand;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use send_authenticate::send_authenticate;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast, mpsc};
use tonic::Streaming;
use wallguard_common::protobuf::wallguard_commands::{
    ClientMessage, ServerMessage, server_message,
};

mod await_authorization;
mod command;
mod commands;
mod post_startup;
mod send_authenticate;

pub(crate) type InboundStream = Arc<Mutex<Streaming<ServerMessage>>>;
pub(crate) type OutboundStream = Arc<Mutex<mpsc::Sender<ClientMessage>>>;

#[derive(Debug, Clone)]
pub struct ControlChannel {
    context: Context,
    terminate: broadcast::Sender<()>,
}

impl ControlChannel {
    pub fn new(context: Context, code: String) -> Self {
        let (terminate, _) = broadcast::channel(1);

        tokio::spawn(stream_wrapper(
            context.clone(),
            code.clone(),
            terminate.subscribe(),
        ));

        Self { context, terminate }
    }

    pub async fn terminate(&self) {
        let mut manager = self.context.transmission_manager.lock().await;

        manager.terminate_packet_capture();
        manager.terminate_resource_monitoring();
        manager.terminate_sysconfig_monitoring();

        drop(manager);

        let _ = self.terminate.send(());
    }
}

async fn stream_wrapper(
    context: Context,
    installation_code: String,
    mut terminate: broadcast::Receiver<()>,
) {
    tokio::select! {
        _ = terminate.recv() => {}
        result = control_stream(context.clone(), &installation_code) => {
            if let Err(err) = result {
                Daemon::on_error(context.daemon, err.to_str()).await;
            }
        }
    };
}

async fn control_stream(context: Context, installation_code: &str) -> Result<(), Error> {
    let (outbound, receiver) = mpsc::channel(64);
    let inbound = context.server.request_control_channel(receiver).await?;

    let inbound = Arc::new(Mutex::new(inbound));
    let outbound = Arc::new(Mutex::new(outbound));

    match await_authorization(
        inbound.clone(),
        outbound.clone(),
        context.client_data.clone(),
        installation_code,
    )
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

    // Clone the outbound stream to keep it alive—closing it signals
    // an error to the server, which closes the connection.
    send_authenticate(outbound.clone()).await?;

    tokio::spawn(post_startup(context.clone()));

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
            server_message::Message::EnableNetworkMonitoringCommand(value) => {
                let cmd = EnableNetworkMonitoringCommand::new(context.clone(), value);

                if let Err(err) = cmd.execute().await {
                    log::error!(
                        "EnableNetworkMonitoringCommand execution failed: {}",
                        err.to_str()
                    );
                }
            }
            server_message::Message::EnableConfigurationMonitoringCommand(value) => {
                let cmd = EnableConfigurationMonitoringCommand::new(context.clone(), value);

                if let Err(err) = cmd.execute().await {
                    log::error!(
                        "EnableConfigurationMonitoringCommand execution failed: {}",
                        err.to_str()
                    );
                }
            }
            server_message::Message::EnableTelemetryMonitoringCommand(value) => {
                let cmd = EnableTelemetryMonitoringCommand::new(context.clone(), value);

                if let Err(err) = cmd.execute().await {
                    log::error!(
                        "EnableTelemetryMonitoringCommand execution failed: {}",
                        err.to_str()
                    );
                }
            }
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
            server_message::Message::CreateFilterRule(rule) => {
                let cmd = CreateFilterRuleCommand::new(rule, context.clone());

                if let Err(err) = cmd.execute().await {
                    log::error!("CreateFilterRuleCommand execution failed: {}", err.to_str());
                }
            }
            server_message::Message::CreateNatRule(rule) => {
                let cmd = CreateNatRuleCommand::new(rule, context.clone());

                if let Err(err) = cmd.execute().await {
                    log::error!("CreateNatRuleCommand execution failed: {}", err.to_str());
                }
            }
            server_message::Message::CreateAlias(alias) => {
                let cmd = CreateAliasCommand::new(alias, context.clone());

                if let Err(err) = cmd.execute().await {
                    log::error!("CreateAliasCommand execution failed: {}", err.to_str());
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
                _ = Storage::delete_value(Secret::AppId).await;
                _ = Storage::delete_value(Secret::AppSecret).await;
                // Gracefuly transition to IDLE state
                todo!();
            }
            server_message::Message::OpenRemoteDesktopSessionCommand(token) => {
                let cmd = OpenRemoteDesktopSessionCommand::new(context.clone(), token);

                if let Err(err) = cmd.execute().await {
                    log::error!(
                        "OpenRemoteDesktopSessionCommand execution failed: {}",
                        err.to_str()
                    );
                }
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
