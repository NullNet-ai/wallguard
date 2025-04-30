use crate::{
    app_context::AppContext, grpc_server::AuthHandler, proto::wallguard::HeartbeatResponse,
};

use super::auth_stream::AuthStream;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::collections::HashMap;
use tokio::sync::mpsc;

pub struct Manager {
    connected_clients: HashMap<String, AuthStream>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            connected_clients: HashMap::new(),
        }
    }

    pub async fn is_client_connected(&self, device_id: &str) -> bool {
        self.connected_clients.contains_key(device_id)
    }

    pub async fn on_client_connected(
        &mut self,
        device_id: String,
        auth_handler: AuthHandler,
        context: AppContext,
        tx: mpsc::Sender<Result<HeartbeatResponse, tonic::Status>>,
    ) -> Result<(), Error> {
        if self.connected_clients.contains_key(&device_id) {
            return Err(format!(
                "Client with device id {device_id} is already connected"
            ))
            .handle_err(location!());
        }

        let stream = AuthStream::new(auth_handler, context, tx);
        let _ = self.connected_clients.insert(device_id, stream);

        Ok(())
    }

    pub fn on_client_disconnected(&mut self, device_id: &str) {
        let _ = self.connected_clients.remove(device_id);
    }

    pub async fn force_heartbeat(&mut self, device_id: &str) -> Result<(), Error> {
        let Some(stream) = self.connected_clients.get_mut(device_id) else {
            return Err(format!("No client connected by device id {device_id}"))
                .handle_err(location!());
        };

        stream.force_update().await?;

        Ok(())
    }
}
