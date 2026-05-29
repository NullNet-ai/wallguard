use generated::store_service_client::StoreServiceClient;
use nullnet_liberror::{Error, ErrorHandler, Location, location};

mod db_tables;
mod generated;
mod models;
mod operations;

pub use models::*;
use tonic::transport::{Channel, ClientTlsConfig};

#[derive(Debug, Clone)]
pub struct Datastore {
    inner: StoreServiceClient<Channel>,
}

impl Datastore {
    pub async fn new() -> Result<Self, Error> {
        let host = read_host_value_from_env(String::from("127.0.0.1"));
        let port = read_port_value_from_env(6000);
        let tls = real_tls_value_from_env(false);

        let channel = connect(host.as_str(), port, tls)?;
        let inner: StoreServiceClient<Channel> =
            StoreServiceClient::new(channel).max_decoding_message_size(50 * 1024 * 1024);

        Ok(Self { inner })
    }
}

fn read_host_value_from_env(default: String) -> String {
    std::env::var("DATASTORE_HOST").unwrap_or_else(|err| {
        log::warn!("Failed to read 'DATASTORE_HOST' env var: {err}. Using default value ...");
        default
    })
}

fn read_port_value_from_env(default: u16) -> u16 {
    match std::env::var("DATASTORE_PORT") {
        Ok(value) => value.parse::<u16>().unwrap_or_else(|err| {
            log::warn!(
                "Failed to parse 'DATASTORE_PORT' ({value}) as u16: {err}. Using default value ..."
            );
            default
        }),
        Err(err) => {
            log::warn!("Failed to read 'DATASTORE_PORT' env var: {err}. Using default value ...");
            default
        }
    }
}

fn real_tls_value_from_env(default: bool) -> bool {
    match std::env::var("DATASTORE_TLS") {
        Ok(value) => value.to_lowercase() == "true",
        Err(err) => {
            log::warn!("Failed to read 'DATASTORE_TLS' env var: {err}. Using default value ...");
            default
        }
    }
}

fn connect(host: &str, port: u16, tls: bool) -> Result<Channel, Error> {
    let protocol = if tls { "https" } else { "http" };

    let mut endpoint = Channel::from_shared(format!("{protocol}://{host}:{port}"))
        .handle_err(location!())?
        .connect_timeout(std::time::Duration::from_secs(10));

    if tls {
        endpoint = endpoint
            .tls_config(ClientTlsConfig::new().with_native_roots())
            .handle_err(location!())?;
    }

    // connect_lazy defers the TCP handshake until first use and automatically
    // reconnects when the datastore restarts (tower detects the broken h2 connection
    // and re-dials on the next request).
    Ok(endpoint.connect_lazy())
}
