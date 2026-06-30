use wallguard_common::protobuf::wallguard_cli::Connected;
use wallguard_common::protobuf::wallguard_cli::Error;
use wallguard_common::protobuf::wallguard_cli::Status;
use wallguard_common::protobuf::wallguard_cli::status::State;

use crate::control_channel::ControlChannel;
use std::fmt;

#[derive(Debug, Clone, Default)]
pub enum DaemonState {
    #[default]
    Idle,
    Connecting,
    Connected(Box<ControlChannel>),
    Error(String),
}

impl DaemonState {
    pub(crate) async fn into_status(self) -> Status {
        let state = match self {
            DaemonState::Idle => State::Idle(()),
            DaemonState::Connecting => State::Connecting(()),
            DaemonState::Connected(control_channel) => {
                let context = control_channel.get_context();
                let data = Connected {
                    device_id: context.token_provider.device_id().await,
                    device_uuid: Some(context.client_data.uuid.clone()),
                };
                State::Connected(data)
            }
            DaemonState::Error(message) => {
                let data = Error { message };
                State::Error(data)
            }
        };

        Status { state: Some(state) }
    }
}

impl fmt::Display for DaemonState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DaemonState::Idle => write!(f, "Idle"),
            DaemonState::Connected(_) => write!(f, "Connected"),
            DaemonState::Error(_) => write!(f, "Error"),
            DaemonState::Connecting => write!(f, "Connecting"),
        }
    }
}
