use crate::proto::wallguard::wall_guard_server::{WallGuard, WallGuardServer};
use crate::proto::wallguard::{Empty, Packets};
use std::net::ToSocketAddrs;
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tonic::{Request, Response, Status};

const ADDR: &str = "0.0.0.0";
const PORT: u16 = 50051;

pub async fn run_grpc_server(tx: async_channel::Sender<Packets>) {
    let addr = format!("{ADDR}:{PORT}")
        .to_socket_addrs()
        .expect("Failed to resolve address")
        .next()
        .expect("Failed to get address");

    let cert =
        std::fs::read_to_string("./tls/wallmon.pem").expect("Failed to read certificate file");
    let key = std::fs::read_to_string("./tls/wallmon-key.pem").expect("Failed to read key file");
    let identity = Identity::from_pem(cert, key);

    Server::builder()
        .tls_config(ServerTlsConfig::new().identity(identity))
        .expect("Failed to set up TLS")
        .add_service(WallGuardServer::new(WallGuardImpl { tx }))
        .serve(addr)
        .await
        .expect("Failed to start gRPC server");
}

struct WallGuardImpl {
    tx: async_channel::Sender<Packets>,
}

#[tonic::async_trait]
impl WallGuard for WallGuardImpl {
    async fn handle_packets(&self, request: Request<Packets>) -> Result<Response<Empty>, Status> {
        self.tx
            .try_send(request.into_inner())
            .map_err(|_| Status::internal("Failed to send packets to workers"))?;

        Ok(Response::new(Empty {}))
    }
}
