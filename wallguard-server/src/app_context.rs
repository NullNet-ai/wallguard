use crate::{
    client_stream::Manager,
    datastore::DatastoreWrapper,
    tunnel::{TunnelServer, monitor_idle_profiles},
};
use nullnet_liberror::Error;
use std::{sync::Arc, time::Duration};
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

        tokio::spawn(monitor_idle_profiles(
            tunnel.clone(),
            Duration::from_secs(60 * 30),
        ));

        Ok(Self {
            datastore,
            tunnel,
            clients_manager,
        })
    }
}
