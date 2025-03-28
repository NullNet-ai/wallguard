use std::{env, net::SocketAddr, time::Duration};

pub struct Config {
    pub addr: SocketAddr,
    pub idle_channel_timeout: Duration,
}

impl Config {
    pub fn from_env() -> Self {
        let addr = format!("0.0.0.0:{}", read_port_from_env(9000))
            .parse::<SocketAddr>()
            .unwrap_or_else(|_| {
                log::error!("Failed to parse address. Using default address 0.0.0.0:8080...");
                "0.0.0.0:8080".parse().unwrap()
            });

        let idle_channel_timeout = read_duration_from_env(60 * 5);

        Config {
            addr,
            idle_channel_timeout,
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
        log::error!(
            "Failed to read 'TUNNEL_PORT' env var. Using default port {}...",
            default_port
        );
        default_port.to_string()
    });

    port.parse::<u16>().unwrap_or_else(|_| {
        log::error!(
            "Failed to parse 'TUNNEL_PORT'. Using default port {}...",
            default_port
        );
        default_port
    })
}

/// Reads a idle channel timeout (in seconds) from an environment variable.
/// Defaults to `default_secs` if the variable is missing or invalid.
fn read_duration_from_env(default_secs: u64) -> Duration {
    let seconds = env::var("IDLE_CHANNEL_TIMEOUT")
        .ok()
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or_else(|| {
            log::error!(
                "Failed to read 'IDLE_CHANNEL_TIMEOUT' or invalid value. Using default {} seconds...", default_secs
            );
            default_secs
        });

    Duration::from_secs(seconds)
}

impl From<Config> for nullnet_libtunnel::ServerConfig {
    fn from(value: Config) -> Self {
        Self {
            addr: value.addr,
            idle_channels_timeout: value.idle_channel_timeout,
        }
    }
}
