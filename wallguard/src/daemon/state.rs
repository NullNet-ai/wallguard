use super::wallguard_cli::status::State;
use super::wallguard_cli::Authorization;
use super::wallguard_cli::Connected;
use super::wallguard_cli::Error;
use super::wallguard_cli::Idle;
use super::wallguard_cli::Status;
use crate::utilities;
use std::fmt;



#[derive(Debug, Clone)]
pub enum DaemonState {
    Idle(u64),
    Authorization(u64, String),
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
            DaemonState::Authorization(timestamp, _) => {
                let data = Authorization { timestamp };
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
            DaemonState::Idle(ts) => {
                let datetime = utilities::time::timestamp_to_iso_string(*ts as i64);
                write!(f, "Idle since {}", datetime)
            }
            DaemonState::Authorization(ts, org) => {
                let datetime = utilities::time::timestamp_to_iso_string(*ts as i64);
                write!(
                    f,
                    "Awaiting authorization for organization '{}' since {}",
                    org, datetime
                )
            }
            DaemonState::Connected(ts, org) => {
                let datetime = utilities::time::timestamp_to_iso_string(*ts as i64);
                write!(f, "Connected to organization '{}' since {}", org, datetime)
            }
            DaemonState::Error(ts, errmsg) => {
                let datetime = utilities::time::timestamp_to_iso_string(*ts as i64);
                write!(f, "Error occurred at {}: '{}'", datetime, errmsg)
            }
        }
    }
}
