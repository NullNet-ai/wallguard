use crate::datastore::DatastoreWrapper;
use crate::proto::wallguard::wall_guard_server::{WallGuard, WallGuardServer};
use crate::proto::wallguard::{
    Authentication, ConfigSnapshot, Empty, HeartbeatRequest, LoginRequest, Packets, SetupRequest,
};
use libdatastore::Token;
use std::net::ToSocketAddrs;
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tonic::{Request, Response, Status};

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

    let cert =
        std::fs::read_to_string("./tls/wallmon.pem").expect("Failed to read certificate file");
    let key = std::fs::read_to_string("./tls/wallmon-key.pem").expect("Failed to read key file");
    let identity = Identity::from_pem(cert, key);

    Server::builder()
        .tls_config(ServerTlsConfig::new().identity(identity))
        .expect("Failed to set up TLS")
        .add_service(WallGuardServer::new(WallGuardImpl { tx, datastore }))
        .serve(addr)
        .await
        .expect("Failed to start gRPC server");
}

struct WallGuardImpl {
    tx: async_channel::Sender<Packets>,
    datastore: Option<DatastoreWrapper>,
}

#[tonic::async_trait]
impl WallGuard for WallGuardImpl {
    async fn heartbeat(
        &self,
        _request: Request<HeartbeatRequest>,
    ) -> Result<Response<Empty>, Status> {
        // TODO: Update last heartbeat
        Ok(Response::new(Empty {}))
    }

    async fn setup(&self, request: Request<SetupRequest>) -> Result<Response<Empty>, Status> {
        let datastore = self
            .datastore
            .as_ref()
            .ok_or_else(|| Status::internal("Datastore is unavailable"))?;

        let remote_address = request
            .remote_addr()
            .map_or_else(|| "Unknown".to_string(), |addr| addr.ip().to_string());

        let setup_request = request.into_inner();

        let jwt_token = setup_request
            .auth
            .ok_or_else(|| Status::internal("Unauthorized request"))?
            .token;

        let token_info =
            Token::from_jwt(&jwt_token).map_err(|e| Status::internal(e.to_string()))?;

        let response = datastore
            .device_setup(
                jwt_token,
                token_info.account.device.id,
                setup_request.device_version,
                setup_request.device_uuid,
                setup_request.hostname,
                remote_address,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        println!("{:?}", response);

        Ok(Response::new(Empty {}))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<Authentication>, Status> {
        let Some(datastore) = self.datastore.as_ref() else {
            return Err(Status::internal("Datastore is unavailable"));
        };

        let login_request = request.into_inner();

        let token = datastore
            .login(login_request.app_id, login_request.app_secret)
            .await
            .map_err(|e| Status::internal(format!("Datastore login failed: {e:?}")))?;

        if token.is_empty() {
            return Err(Status::internal(
                "Datastore login failed: Wrong credentials",
            ));
        }

        Ok(Response::new(Authentication { token }))
    }

    async fn handle_packets(&self, request: Request<Packets>) -> Result<Response<Empty>, Status> {
        self.tx
            .try_send(request.into_inner())
            .map_err(|_| Status::internal("Failed to send packets to workers"))?;

        Ok(Response::new(Empty {}))
    }

    async fn handle_config(
        &self,
        request: Request<ConfigSnapshot>,
    ) -> Result<Response<Empty>, Status> {
        let snapshot = request.into_inner();

        for file in &snapshot.files {
            let name = &file.filename;
            let len = file.contents.len();
            println!("Received file {name} of len {len} bytes");
        }

        println!("---");

        Ok(Response::new(Empty {}))
    }
}
