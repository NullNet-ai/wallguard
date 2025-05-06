use crate::{
    client_stream::Manager,
    datastore::{DatastoreWrapper, DatastoreWrapperExperimental},
    tunnel::TunnelServer,
};
use nullnet_liberror::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppContext {
    pub datastore: DatastoreWrapper,
    pub datastore_exp: Option<DatastoreWrapperExperimental>,
    pub tunnel: Arc<Mutex<TunnelServer>>,
    pub clients_manager: Arc<Mutex<Manager>>,
}

impl AppContext {
    pub async fn new() -> Result<Self, Error> {
        let datastore = DatastoreWrapper::new().await?;
        let clients_manager = Arc::new(Mutex::new(Manager::new()));
        let tunnel = Arc::new(Mutex::new(TunnelServer::new()));

        let use_expr_datastore = match std::env::var("USE_EXPERIMENTAL_DATASTORE") {
            Ok(value) => value.to_lowercase() == "true",
            Err(err) => {
                log::warn!(
                    "Failed to read 'USE_EXPERIMENTAL_DATASTORE' env var: {err}. Using default value ..."
                );
                false
            }
        };

        let datastore_exp = if use_expr_datastore {
            Some(DatastoreWrapperExperimental::new().await?)
        } else {
            None
        };

        Ok(Self {
            datastore,
            datastore_exp,
            tunnel,
            clients_manager,
        })
    }
}
