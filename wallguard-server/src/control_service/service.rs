use crate::protocol::wallguard_service::wall_guard_server::WallGuardServer;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::net::SocketAddr;
use tonic::transport::Server;

#[derive(Debug)]
pub struct WallGuardService;

impl WallGuardService {
    pub async fn serve(addr: SocketAddr) -> Result<(), Error> {
        let service = WallGuardService {};

        Server::builder()
            .add_service(WallGuardServer::new(service))
            .serve(addr)
            .await
            .handle_err(location!())?;

        Ok(())
    }
}
