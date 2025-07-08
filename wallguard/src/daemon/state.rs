use super::wallguard_cli::status::State;
use super::wallguard_cli::Connected;
use super::wallguard_cli::Error;
use super::wallguard_cli::Status;
use crate::control_channel::ControlChannel;
use std::fmt;

#[derive(Debug, Clone)]
pub enum DaemonState {
    Idle,
    Connected(ControlChannel),
    Error(String),
}

impl Default for DaemonState {
    fn default() -> Self {
        Self::Idle
    }
}

impl Into<Status> for DaemonState {
    fn into(self) -> Status {
        let state = match self {
            DaemonState::Idle => State::Idle(()),
            DaemonState::Connected(_) => {
                let data = Connected {};
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
        }
    }
}
