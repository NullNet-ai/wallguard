use crate::proto::name::name_server::NameServer;
use crate::grpc_impl::NameImpl;
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
        .add_service(NameServer::new(NameImpl))
        .serve(addr)
        .await
        .unwrap();
}
