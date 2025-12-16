use std::time::Duration;

use nullnet_liberror::{Error, ErrorHandler, Location, location};

use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tonic::Status;
use tonic::Streaming;
use wallguard_common::protobuf::wallguard_commands::ExecuteCliCommandRequest;
use wallguard_common::protobuf::wallguard_commands::ExecuteCliCommandResponse;
use wallguard_common::protobuf::wallguard_models::Alias;
use wallguard_common::protobuf::wallguard_models::FilterRule;
use wallguard_common::protobuf::wallguard_models::NatRule;

use crate::app_context::AppContext;
use crate::orchestrator::control_stream::control_stream;
use wallguard_common::protobuf::wallguard_commands::AuthenticationData;
use wallguard_common::protobuf::wallguard_commands::ClientMessage;
use wallguard_common::protobuf::wallguard_commands::ServerMessage;
use wallguard_common::protobuf::wallguard_commands::SshSessionData;
use wallguard_common::protobuf::wallguard_commands::UiSessionData;
use wallguard_common::protobuf::wallguard_commands::server_message::Message;

pub(crate) type OutboundStream = mpsc::Sender<Result<ServerMessage, Status>>;
pub(crate) type InboundStream = Streaming<ClientMessage>;

#[derive(Debug)]
pub struct Instance {
    pub(crate) device_id: String,
    pub(crate) instance_id: String,
    pub(crate) outbound: OutboundStream,
    pub(crate) channel: broadcast::Sender<ExecuteCliCommandResponse>,
}

impl Instance {
    pub fn new(
        device_id: String,
        instance_id: String,
        inbound: InboundStream,
        outbound: OutboundStream,
        context: AppContext,
    ) -> Self {
        let (channel, _) = broadcast::channel(64);

        tokio::spawn(control_stream(
            device_id.clone(),
            instance_id.clone(),
            inbound,
            outbound.clone(),
            context,
            channel.clone(),
        ));

        Self {
            device_id,
            instance_id,
            outbound,
            channel,
        }
    }

    pub async fn authorize(&mut self, data: AuthenticationData) -> Result<(), Error> {
        log::debug!(
            "Authorizing Device ID {}, Instance {}",
            self.device_id,
            self.instance_id
        );

        let message = ServerMessage {
            message: Some(Message::DeviceAuthorizedMessage(data)),
        };

        self.outbound
            .send(Ok(message))
            .await
            .handle_err(location!())?;

        Ok(())
    }

    pub async fn _deauthorize(&mut self) -> Result<(), Error> {
        log::debug!(
            "Deauthorizing Device ID {}, Instance {}",
            self.device_id,
            self.instance_id
        );

        let message = ServerMessage {
            message: Some(Message::DeviceDeauthorizedMessage(())),
        };

        self.outbound
            .send(Ok(message))
            .await
            .handle_err(location!())?;

        Ok(())
    }

    pub async fn enable_network_monitoring(&self, enable: bool) -> Result<(), Error> {
        log::info!(
            "Sending EnableNetworkMonitoringCommand to the client with device ID {}, Instance {}",
            self.device_id,
            self.instance_id
        );

        let message = ServerMessage {
            message: Some(Message::EnableNetworkMonitoringCommand(enable)),
        };

        self.outbound
            .send(Ok(message))
            .await
            .handle_err(location!())
    }

    pub async fn enable_telemetry_monitoring(&self, enable: bool) -> Result<(), Error> {
        log::info!(
            "Sending EnableTelemetryMonitoringCommand to the client with device ID {}, Instance {}",
            self.device_id,
            self.instance_id
        );

        let message = ServerMessage {
            message: Some(Message::EnableTelemetryMonitoringCommand(enable)),
        };

        self.outbound
            .send(Ok(message))
            .await
            .handle_err(location!())
    }

    pub async fn enable_configuration_monitoring(&self, enable: bool) -> Result<(), Error> {
        log::info!(
            "Sending EnableConfigurationMonitoringCommand to the client with device ID {}, Instance {}",
            self.device_id,
            self.instance_id
        );

        let message = ServerMessage {
            message: Some(Message::EnableConfigurationMonitoringCommand(enable)),
        };

        self.outbound
            .send(Ok(message))
            .await
            .handle_err(location!())
    }

    pub async fn request_ssh_session(
        &self,
        tunnel_token: impl Into<String>,
        public_key: impl Into<String>,
    ) -> Result<(), Error> {
        log::info!(
            "Sending OpenSshSessionCommandto to the client with device ID {}, Instance {}",
            self.device_id,
            self.instance_id
        );

        let ssh_session_data = SshSessionData {
            tunnel_token: tunnel_token.into(),
            public_key: public_key.into(),
        };

        let message: ServerMessage = ServerMessage {
            message: Some(Message::OpenSshSessionCommand(ssh_session_data)),
        };

        self.outbound
            .send(Ok(message))
            .await
            .handle_err(location!())
    }

    pub async fn request_tty_session(&self, tunnel_token: impl Into<String>) -> Result<(), Error> {
        log::info!(
            "Sending OpenTtySessionCommand to the client with device ID {}, Instance {}",
            self.device_id,
            self.instance_id
        );

        let message = ServerMessage {
            message: Some(Message::OpenTtySessionCommand(tunnel_token.into())),
        };

        self.outbound
            .send(Ok(message))
            .await
            .handle_err(location!())
    }

    pub async fn request_ui_session(
        &self,
        tunnel_token: impl Into<String>,
        local_addr: impl Into<String>,
        local_port: u32,
        protocol: impl Into<String>,
    ) -> Result<(), Error> {
        log::info!(
            "Sending OpenUiSessionCommand to the client with device ID {}, Instance {}",
            self.device_id,
            self.instance_id
        );

        let ui_session_data = UiSessionData {
            tunnel_token: tunnel_token.into(),
            protocol: protocol.into(),
            local_addr: local_addr.into(),
            local_port,
        };

        let message = ServerMessage {
            message: Some(Message::OpenUiSessionCommand(ui_session_data)),
        };

        self.outbound
            .send(Ok(message))
            .await
            .handle_err(location!())
    }

    pub async fn request_remote_desktop_session(
        &self,
        tunnel_token: impl Into<String>,
    ) -> Result<(), Error> {
        log::info!(
            "Sending OpenRemoteDesktopSessionCommand to the client with device ID {}, Instance {}",
            self.device_id,
            self.instance_id
        );

        let message = ServerMessage {
            message: Some(Message::OpenRemoteDesktopSessionCommand(
                tunnel_token.into(),
            )),
        };

        self.outbound
            .send(Ok(message))
            .await
            .handle_err(location!())
    }

    pub async fn create_filter_rule(&self, rule: FilterRule) -> Result<(), Error> {
        log::info!(
            "Sending CreateFilterRule to the client with device ID {}, Instance {}",
            self.device_id,
            self.instance_id
        );

        let message = ServerMessage {
            message: Some(Message::CreateFilterRule(rule)),
        };

        self.outbound
            .send(Ok(message))
            .await
            .handle_err(location!())
    }

    pub async fn create_nat_rule(&self, rule: NatRule) -> Result<(), Error> {
        log::info!(
            "Sending CreateNatRule to the client with device ID {}, Instance {}",
            self.device_id,
            self.instance_id
        );

        let message = ServerMessage {
            message: Some(Message::CreateNatRule(rule)),
        };

        self.outbound
            .send(Ok(message))
            .await
            .handle_err(location!())
    }

    pub async fn create_alias(&self, alias: Alias) -> Result<(), Error> {
        log::info!(
            "Sending CreateAlias to the client with device ID {}, Instance {}",
            self.device_id,
            self.instance_id
        );

        let message = ServerMessage {
            message: Some(Message::CreateAlias(alias)),
        };

        self.outbound
            .send(Ok(message))
            .await
            .handle_err(location!())
    }

    pub async fn execute_cli_command(
        &self,
        request: ExecuteCliCommandRequest,
        timeout: Duration,
    ) -> Result<ExecuteCliCommandResponse, Error> {
        log::info!(
            "Sending ExecuteCliCommandRequest to the client with device ID {}, Instance {}",
            self.device_id,
            self.instance_id
        );

        let request_id = request.request_unique_id.clone();

        let message = ServerMessage {
            message: Some(Message::ExecuteCliCommandRequest(request)),
        };

        let mut channel = self.channel.subscribe();

        self.outbound
            .send(Ok(message))
            .await
            .handle_err(location!())?;

        let result = tokio::time::timeout(timeout, async {
            loop {
                let Ok(response) = channel.recv().await else {
                    return None;
                };

                if response.request_unique_id == request_id {
                    return Some(response);
                }
            }
        })
        .await;

        match result {
            Ok(retval) => retval
                .ok_or("Failed to receive response")
                .handle_err(location!()),
            Err(_) => Err("Request timed out").handle_err(location!()),
        }
    }
}
