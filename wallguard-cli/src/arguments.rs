use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Parser)]
#[command(name = "wallguard-cli")]
#[command(about = "CLI client for Wallguard service", long_about = None)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, ValueEnum, Serialize, Deserialize)]
pub enum Platform {
    Pfsense,
    Opnsense,
    NfTables,
    Generic,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Platform::Pfsense => "pfsense",
            Platform::Opnsense => "opnsense",
            Platform::Generic => "generic",
            Platform::NfTables => "nftables",
        };
        write!(f, "{name}")
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Get current system status
    Status,

    /// Get monitoring capabilities
    Capabilities,

    /// Join an organization
    Join {
        /// Installation Code
        installation_code: String,
    },

    /// Leave the current organization
    Leave,

    /// Retry connecting after the agent gave up (state: ERROR)
    Reconnect,

    /// Start the service with optional configuration
    Start {
        /// URL for the control channel
        #[arg(long)]
        control_channel_url: Option<String>,

        /// Target platform
        #[arg(long, value_enum)]
        platform: Option<Platform>,

        /// Maximum number of packets per batch sent to the server
        #[arg(long)]
        batch_size: Option<usize>,
    },

    /// Get agent version
    Version,

    /// Stop the running service
    Stop,

    /// Restart the running service, preserving its current configuration
    Restart,

    /// Update WallGuard to the latest released version
    Update {
        /// Only report whether a newer version is available; do not install it
        #[arg(long)]
        check: bool,
    },
}
