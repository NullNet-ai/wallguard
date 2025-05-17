use crate::app_context::AppContext;
use crate::protocol::wallguard_service::wall_guard_server::WallGuardServer;

use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::net::SocketAddr;
use tonic::transport::Server;

#[derive(Debug)]
pub struct WallGuardService {
    pub(crate) context: AppContext,
}

impl WallGuardService {
    pub fn new(context: AppContext) -> Self {
        Self { context }
    }

    pub async fn serve(self, addr: SocketAddr) -> Result<(), Error> {
        Server::builder()
            .add_service(WallGuardServer::new(self))
            .serve(addr)
            .await
            .handle_err(location!())?;

        Ok(())
    }
}
