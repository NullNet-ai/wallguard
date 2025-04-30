use crate::{client_stream::Manager, datastore::DatastoreWrapper, tunnel::TunnelServer};
use nullnet_liberror::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppContext {
    pub datastore: DatastoreWrapper,
    pub tunnel: Arc<Mutex<TunnelServer>>,
    pub clients_manager: Arc<Mutex<Manager>>,
}

impl AppContext {
    pub async fn new() -> Result<Self, Error> {
        let datastore = DatastoreWrapper::new().await?;
        let clients_manager = Arc::new(Mutex::new(Manager::new()));
        let tunnel = Arc::new(Mutex::new(TunnelServer::new()));

        Ok(Self {
            datastore,
            tunnel,
            clients_manager,
        })
    }
}
