use clap::{Parser, Subcommand, ValueEnum};
use std::fmt;

#[derive(Debug, Parser)]
#[command(name = "wallguard-cli")]
#[command(about = "CLI client for Wallguard service", long_about = None)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, ValueEnum)]
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

    /// Start the service with optional configuration
    Start {
        /// Host address for the control channel
        #[arg(long)]
        control_channel_host: Option<String>,

        /// Port for the control channel
        #[arg(long)]
        control_channel_port: Option<u16>,

        /// Target platform
        #[arg(long, value_enum, default_value_t = Platform::Generic)]
        platform: Platform,
    },

    /// Stop the running service
    Stop,
}
