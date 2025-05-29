use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "wallguard-cli")]
#[command(about = "CLI client for Wallguard service", long_about = None)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Get current system status
    Status,

    /// Get monitoring capabilities
    Capabilities,

    /// Join an organization
    Join {
        /// Organization ID
        org_id: String,
    },

    /// Leave the current organization
    Leave,
}
