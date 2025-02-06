use nullnet_libdatastore::{DatastoreClient, DatastoreConfig};

mod features;

#[derive(Debug, Clone)]
pub struct DatastoreWrapper {
    inner: DatastoreClient,
}

impl DatastoreWrapper {
    pub fn new() -> Self {
        let config = DatastoreConfig::from_env();
        let inner = DatastoreClient::new(config);
        Self { inner }
    }
}
