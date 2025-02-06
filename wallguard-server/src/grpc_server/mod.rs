use crate::{
    datastore::DatastoreWrapper,
    proto::wallguard::{wall_guard_server::WallGuardServer, Packets},
};
use server::WallGuardImpl;
use std::net::ToSocketAddrs;
use tonic::transport::Server;

mod common;
mod endpoints;
mod server;

const ADDR: &str = "0.0.0.0";
const PORT: u16 = 50051;

pub async fn run_grpc_server(
    tx: async_channel::Sender<Packets>,
    datastore: Option<DatastoreWrapper>,
) {
    let addr = format!("{ADDR}:{PORT}")
        .to_socket_addrs()
        .expect("Failed to resolve address")
        .next()
        .expect("Failed to get address");

    // let cert =
    //     std::fs::read_to_string("./tls/wallmon.pem").expect("Failed to read certificate file");
    // let key = std::fs::read_to_string("./tls/wallmon-key.pem").expect("Failed to read key file");
    // let identity = Identity::from_pem(cert, key);

    Server::builder()
        // .tls_config(ServerTlsConfig::new().identity(identity))
        // .expect("Failed to set up TLS")
        .add_service(WallGuardServer::new(WallGuardImpl { tx, datastore }))
        .serve(addr)
        .await
        .expect("Failed to start gRPC server");
}
