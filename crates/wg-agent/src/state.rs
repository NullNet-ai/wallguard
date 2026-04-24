use wg_shared::types::Feature;

/// Runtime daemon state. Transitions are one-way (no Error state).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonState {
    /// Device cert absent; waiting for `wg-cli enroll`.
    Provisioning,
    /// Enrolled but refusing to retry (e.g. server rejected protocol version).
    Idle(IdleReason),
    /// Attempting to connect to the server; uses exponential backoff.
    Connecting,
    /// Connected and processing commands. Carries negotiated feature set.
    Connected { features: Vec<Feature> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdleReason {
    VersionRejected { min_required: u32 },
}

impl DaemonState {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Provisioning    => "provisioning",
            Self::Idle(_)         => "idle",
            Self::Connecting      => "connecting",
            Self::Connected { .. } => "connected",
        }
    }

    /// Maps to the `DaemonState` proto enum value in `cli.proto`.
    pub fn to_proto_i32(&self) -> i32 {
        match self {
            Self::Provisioning    => 0,
            Self::Idle(_)         => 1,
            Self::Connecting      => 2,
            Self::Connected { .. } => 3,
        }
    }
}
