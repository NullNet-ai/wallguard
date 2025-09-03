use std::process::Command;

use nullnet_liberror::{location, Error, ErrorHandler, Location};

pub struct NftablesHostnameParser;

impl NftablesHostnameParser {
    pub fn parse() -> Result<String, Error> {
        let output = Command::new("hostname").output().handle_err(location!())?;

        if output.status.success() {
            let hostname = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(hostname)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("hostname command failed: {}", stderr)).handle_err(location!())
        }
    }
}
