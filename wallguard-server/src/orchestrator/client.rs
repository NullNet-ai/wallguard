use nullnet_liberror::{Error, ErrorHandler, Location, location};

use super::control_stream::control_stream_task;
use crate::orchestrator::Orchestrator;
use crate::orchestrator::control_stream::ControlStream;
use crate::protocol::wallguard_commands::SshSessionData;
use crate::protocol::wallguard_commands::UiSessionData;
use crate::protocol::wallguard_commands::WallGuardCommand;
use crate::protocol::wallguard_commands::wall_guard_command::Command;
use crate::token_provider::TokenProvider;

#[derive(Debug, Clone)]
pub struct Client {
    device_uuid: String,
    control_stream: ControlStream,
}

impl Client {
    pub fn new(
        device_uuid: &str,
        token_provider: TokenProvider,
        control_stream: ControlStream,
        orchestrator: Orchestrator,
    ) -> Self {
        tokio::spawn(control_stream_task(
            device_uuid.into(),
            control_stream.clone(),
            token_provider.clone(),
            orchestrator,
        ));

        Self {
            device_uuid: device_uuid.into(),
            control_stream,
        }
    }

    pub async fn enable_network_monitoring(&self, enable: bool) -> Result<(), Error> {
        log::info!(
            "Sending EnableNetworkMonitoringCommand('{}') to the client with device UUID {}",
            enable,
            self.device_uuid
        );

        let command = WallGuardCommand {
            command: Some(Command::EnableNetworkMonitoringCommand(enable)),
        };

        self.control_stream
            .send(Ok(command))
            .await
            .handle_err(location!())
    }

    pub async fn enable_configuration_monitoring(&self, enable: bool) -> Result<(), Error> {
        log::info!(
            "Sending EnableConfigurationMonitoringCommand('{}') to the client with device UUID {}",
            enable,
            self.device_uuid
        );

        let command = WallGuardCommand {
            command: Some(Command::EnableConfigurationMonitoringCommand(enable)),
        };

        self.control_stream
            .send(Ok(command))
            .await
            .handle_err(location!())
    }

    pub async fn enable_telemetry_monitoring(&self, enable: bool) -> Result<(), Error> {
        log::info!(
            "Sending EnableTelemetryMonitoringCommand('{}') to the client with device UUID {}",
            enable,
            self.device_uuid
        );

        let command = WallGuardCommand {
            command: Some(Command::EnableTelemetryMonitoringCommand(enable)),
        };

        self.control_stream
            .send(Ok(command))
            .await
            .handle_err(location!())
    }

    pub async fn request_ssh_session(
        &self,
        tunnel_token: impl Into<String>,
        public_key: impl Into<String>,
    ) -> Result<(), Error> {
        log::info!(
            "Sending OpenSshSessionCommandto to the client with device UUID {}",
            self.device_uuid
        );

        let ssh_session_data = SshSessionData {
            tunnel_token: tunnel_token.into(),
            public_key: public_key.into(),
        };

        let command = WallGuardCommand {
            command: Some(Command::OpenSshSessionCommand(ssh_session_data)),
        };

        self.control_stream
            .send(Ok(command))
            .await
            .handle_err(location!())
    }

    pub async fn request_tty_session(&self, tunnel_token: impl Into<String>) -> Result<(), Error> {
        log::info!(
            "Sending OpenTtySessionCommand to the client with device UUID {}",
            self.device_uuid
        );

        let command = WallGuardCommand {
            command: Some(Command::OpenTtySessionCommand(tunnel_token.into())),
        };

        self.control_stream
            .send(Ok(command))
            .await
            .handle_err(location!())
    }

    pub async fn request_ui_session(
        &self,
        tunnel_token: impl Into<String>,
        protocol: impl Into<String>,
    ) -> Result<(), Error> {
        log::info!(
            "Sending OpenUiSessionCommand to the client with device UUID {}",
            self.device_uuid
        );

        let ui_session_data = UiSessionData {
            tunnel_token: tunnel_token.into(),
            protocol: protocol.into(),
        };

        let command = WallGuardCommand {
            command: Some(Command::OpenUiSessionCommand(ui_session_data)),
        };

        self.control_stream
            .send(Ok(command))
            .await
            .handle_err(location!())
    }
}
