use std::net::SocketAddr;
use std::str::FromStr;

pub struct ControlServiceConfig {
    pub(crate) addr: SocketAddr,
}

impl Default for ControlServiceConfig {
    fn default() -> Self {
        let addr = SocketAddr::from_str("0.0.0.0:50051").unwrap();
        ControlServiceConfig { addr }
    }
}

impl ControlServiceConfig {
    pub fn from_env() -> Self {
        let host = std::env::var("CONTROL_SERVICE_ADDR").ok();
        let port = std::env::var("CONTROL_SERVICE_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok());

        if let (Some(host), Some(port)) = (host, port)
            && let Ok(addr) = format!("{host}:{port}",).parse::<SocketAddr>()
        {
            return Self { addr };
        }

        Self::default()
    }
}

/// Reads `STORE_TELEMETRY_IN_DATASTORE`. When `false`, connections and system
/// resources reported by agents are accepted but not written to the datastore.
/// Defaults to `true`.
pub fn store_telemetry_in_datastore() -> bool {
    const NAME: &str = "STORE_TELEMETRY_IN_DATASTORE";

    let Ok(raw) = std::env::var(NAME) else {
        return true;
    };

    match raw.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" => true,
        "false" | "0" | "no" => false,
        _ => {
            log::warn!("{NAME} is set to {raw:?}, which is not a valid boolean; using true");
            true
        }
    }
}
