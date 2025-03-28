use crate::{datastore::DatastoreWrapper, tunnel::TunnelServer};
use nullnet_liberror::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppContext {
    pub datastore: DatastoreWrapper,
    pub tunnel: Arc<Mutex<TunnelServer>>,
}

impl AppContext {
    pub async fn new() -> Result<Self, Error> {
        let datastore = DatastoreWrapper::new().await?;

        let tunnel = Arc::new(Mutex::new(TunnelServer::new()));

        Ok(Self { datastore, tunnel })
    }
}
