use crate::grpc_impl::WallGuardImpl;
use crate::proto::wallguard::wall_guard_server::WallGuardServer;
use std::net::ToSocketAddrs;
use tonic::transport::Server;

mod grpc_impl;
mod proto;

#[tokio::main]
async fn main() {
    let addr = "localhost:50051"
        .to_string()
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    Server::builder()
        .add_service(WallGuardServer::new(WallGuardImpl))
        .serve(addr)
        .await
        .unwrap();
}
