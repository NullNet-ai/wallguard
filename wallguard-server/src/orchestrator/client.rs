use nullnet_liberror::{Error, ErrorHandler, Location, location};

use tokio::sync::mpsc;
use tonic::Status;
use tonic::Streaming;

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
pub struct Client {
    uuid: String,
    _org_id: String,
    outbound: OutboundStream,
}

impl Client {
    pub fn new(
        uuid: String,
        org_id: String,
        inbound: InboundStream,
        outbound: OutboundStream,
        context: AppContext,
    ) -> Self {
        tokio::spawn(control_stream(
            uuid.clone(),
            inbound,
            outbound.clone(),
            context,
        ));

        Self {
            uuid,
            outbound,
            _org_id: org_id,
        }
    }

    pub async fn authorize(&mut self, data: AuthenticationData) -> Result<(), Error> {
        log::debug!("Authorizing device {}", self.uuid);

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
        log::debug!("Deauthorizing device {}", self.uuid);

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
            "Sending EnableNetworkMonitoringCommand to the client with device UUID {}",
            self.uuid
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
            "Sending EnableTelemetryMonitoringCommand to the client with device UUID {}",
            self.uuid
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
            "Sending EnableConfigurationMonitoringCommand to the client with device UUID {}",
            self.uuid
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
            "Sending OpenSshSessionCommandto to the client with device UUID {}",
            self.uuid
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
            "Sending OpenTtySessionCommand to the client with device UUID {}",
            self.uuid
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
        protocol: impl Into<String>,
    ) -> Result<(), Error> {
        log::info!(
            "Sending OpenUiSessionCommand to the client with device UUID {}",
            self.uuid
        );

        let ui_session_data = UiSessionData {
            tunnel_token: tunnel_token.into(),
            protocol: protocol.into(),
        };

        let message = ServerMessage {
            message: Some(Message::OpenUiSessionCommand(ui_session_data)),
        };

        self.outbound
            .send(Ok(message))
            .await
            .handle_err(location!())
    }
}
