use crate::daemon::authorization_task::AuthorizationTask;

use super::wallguard_cli::status::State;
use super::wallguard_cli::Authorization;
use super::wallguard_cli::Connected;
use super::wallguard_cli::Error;
use super::wallguard_cli::Idle;
use super::wallguard_cli::Status;
use std::fmt;

#[derive(Debug, Clone)]
pub enum DaemonState {
    Idle(u64),
    Authorization(AuthorizationTask),
    Connected(u64, String),
    Error(u64, String),
}

impl Default for DaemonState {
    fn default() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        Self::Idle(current_timestamp as u64)
    }
}

impl Into<Status> for DaemonState {
    fn into(self) -> Status {
        let state = match self {
            DaemonState::Idle(timestamp) => {
                let data = Idle { timestamp };
                State::Idle(data)
            }
            DaemonState::Authorization(task) => {
                let data = Authorization {
                    timestamp: task.timestamp(),
                };
                State::Authorization(data)
            }
            DaemonState::Connected(timestamp, org_id) => {
                let data = Connected { timestamp, org_id };
                State::Connected(data)
            }
            DaemonState::Error(timestamp, message) => {
                let data = Error { timestamp, message };
                State::Error(data)
            }
        };

        Status { state: Some(state) }
    }
}

impl fmt::Display for DaemonState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DaemonState::Idle(_) => write!(f, "IDLE"),
            DaemonState::Authorization(_) => write!(f, "AUTH"),
            DaemonState::Connected(_, _) => write!(f, "CONN"),
            DaemonState::Error(_, _) => write!(f, "ERR"),
        }
    }
}
