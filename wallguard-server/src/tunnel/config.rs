use std::{env, net::SocketAddr, time::Duration};

pub struct Config {
    pub addr: SocketAddr,
    pub heartbeat_interval: Option<Duration>,
}

impl Config {
    pub fn from_env() -> Self {
        let addr = format!("0.0.0.0:{}", read_port_from_env(9000))
            .parse::<SocketAddr>()
            .unwrap_or_else(|_| {
                eprintln!("Failed to parse address. Using default address 0.0.0.0:8080...");
                "0.0.0.0:8080".parse().unwrap()
            });

        let heartbeat_interval = read_heartbeat_interval_from_env(None);

        Config {
            addr,
            heartbeat_interval,
        }
    }
}

/// Reads the `TUNNEL_PORT` environment variable.
/// If the variable is not set or an error occurs, defaults to the provided `default_port`.
///
/// # Parameters
/// - `default_port`: Default port to use if the environment variable is not set or parsing fails.
fn read_port_from_env(default_port: u16) -> u16 {
    let port = env::var("TUNNEL_PORT").unwrap_or_else(|_| {
        eprintln!(
            "Failed to read 'TUNNEL_PORT' env var. Using default port {}...",
            default_port
        );
        default_port.to_string()
    });

    port.parse::<u16>().unwrap_or_else(|_| {
        eprintln!(
            "Failed to parse 'TUNNEL_PORT'. Using default port {}...",
            default_port
        );
        default_port
    })
}

/// Reads the `HEARTBEAT_INTERVAL` environment variable.
/// If the variable is not set or an error occurs, defaults to the provided `default_heartbeat_interval`.
///
/// # Parameters
/// - `default_heartbeat_interval`: Default heartbeat interval to use if the environment variable is not set or parsing fails.
fn read_heartbeat_interval_from_env(default_heartbeat_interval: Option<u64>) -> Option<Duration> {
    match env::var("HEARTBEAT_INTERVAL") {
        Ok(val) => val.parse::<u64>().ok().map(Duration::from_secs),
        Err(_) => {
            if let Some(default) = default_heartbeat_interval {
                eprintln!("Failed to read 'HEARTBEAT_INTERVAL' env var. Using default heartbeat interval {}...", default);
                Some(Duration::from_secs(default))
            } else {
                eprintln!("Failed to read 'HEARTBEAT_INTERVAL' env var. No heartbeat interval will be set.");
                None
            }
        }
    }
}
