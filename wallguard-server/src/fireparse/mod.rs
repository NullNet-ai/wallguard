mod models;
mod nftables;
mod pfsense;
mod utils;

pub use models::*;
use pfsense::PfSenseParser;
use wallguard_common::{protobuf::wallguard_service::FileSnapshot, wallguard_platform::Platform};

/// Represents possible errors that can occur while parsing firewall configurations.
pub enum FireparseError {
    UnsupportedPlatform(String),
    ParserError(String),
}

/// A generic parser for firewall configuration files.
///
/// This parser determines the correct parsing logic based on the specified platform.
pub struct Parser {}

impl Parser {
    /// Parses a firewall configuration snapshot based on the specified platform.
    ///
    /// # Arguments
    /// * `platform` - The firewall platform (e.g., `Platform::PfSense` or `Platform::OPNsense`).
    /// * `snapshot` - A `Snapshot` representing the firewall configuration state.
    ///
    /// # Returns
    /// * `Ok(Configuration)` - If parsing is successful, returns a `Configuration` struct.
    /// * `Err(FireparseError)` - If the platform is unsupported or the snapshot is invalid..
    pub fn parse(
        platfom: Platform,
        snapshot: Vec<FileSnapshot>,
    ) -> Result<Configuration, FireparseError> {
        match platfom {
            Platform::PfSense => PfSenseParser::parse(snapshot),
            Platform::OpnSense => todo!(),
            Platform::Generic => Err(FireparseError::UnsupportedPlatform(
                "Generic platforms do not support configuration monitoring".into(),
            )),
        }
    }
}
