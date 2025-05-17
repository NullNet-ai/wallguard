use nullnet_libdatastore::{DatastoreClient, DatastoreConfig};
use nullnet_liberror::Error;

mod builders;
mod operations;

#[derive(Debug, Clone)]
pub struct Datastore {
    inner: DatastoreClient,
}

impl Datastore {
    pub async fn new() -> Result<Self, Error> {
        let config = DatastoreConfig::from_env();
        let inner = DatastoreClient::new(config).await?;
        Ok(Self { inner })
    }
}
