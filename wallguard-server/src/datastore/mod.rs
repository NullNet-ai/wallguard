use nullnet_libdatastore::{DatastoreClient, DatastoreConfig, ExperimentalDatastoreClient};
use nullnet_liberror::Error;

mod features;

#[derive(Debug, Clone)]
pub struct DatastoreWrapper {
    inner: DatastoreClient,
}

impl DatastoreWrapper {
    pub async fn new() -> Result<Self, Error> {
        let config = DatastoreConfig::from_env();
        let inner = DatastoreClient::new(config).await?;
        Ok(Self { inner })
    }
}

#[derive(Debug, Clone)]
pub struct DatastoreWrapperExperimental {
    pub(crate) inner: ExperimentalDatastoreClient,
}

impl DatastoreWrapperExperimental {
    pub async fn new() -> Result<Self, Error> {
        let host = std::env::var("DATASTORE_HOST_EXP").unwrap_or_else(|err| {
            log::warn!(
                "Failed to read 'DATASTORE_HOST_EXP' env var: {err}. Using default value ..."
            );
            String::from("rust-grpc.nullnetqa.net")
        });

        let port = match std::env::var("DATASTORE_PORT_EXP") {
            Ok(value) => value.parse::<u16>().unwrap_or_else(|err| {
                log::warn!(
                    "Failed to parse 'DATASTORE_PORT_EXP' ({value}) as u16: {err}. Using default value ..."
                );
                6001
            }),
            Err(err) => {
                log::warn!("Failed to read 'DATASTORE_PORT_EXP' env var: {err}. Using default value ...");
                6001
            }
        };

        let tls = match std::env::var("DATASTORE_TLS_EXP") {
            Ok(value) => value.to_lowercase() == "true",
            Err(err) => {
                log::warn!(
                    "Failed to read 'DATASTORE_TLS_EXP' env var: {err}. Using default value ..."
                );
                false
            }
        };

        let config = DatastoreConfig::new(host, port, tls);
        let inner = ExperimentalDatastoreClient::new(config).await?;

        Ok(Self { inner })
    }
}
