use std::{env, net::SocketAddr};

pub struct Config {
    pub addr: SocketAddr,
}

impl Config {
    pub fn from_env() -> Self {
        let addr = format!("0.0.0.0:{}", read_port_from_env(9000))
            .parse::<SocketAddr>()
            .unwrap_or_else(|_| {
                eprintln!("Failed to parse address. Using default address 0.0.0.0:8080...");
                "0.0.0.0:8080".parse().unwrap()
            });

        Config { addr }
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

impl From<Config> for nullnet_libtunnel::ServerConfig {
    fn from(value: Config) -> Self {
        Self { addr: value.addr }
    }
}
