use crate::client_data::ClientData;
use crate::constants::DISK_SIZE;
use crate::daemon::Daemon;
use crate::data_transmission::dump_dir::DumpDir;
use crate::data_transmission::transmission_manager::TransmissionManager;
use crate::reverse_tunnel::ReverseTunnel;
use crate::server_data::ServerData;
use crate::token_provider::TokenProvider;
use crate::wg_server::WGServer;
use nullnet_liberror::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct Context {
    pub token_provider: TokenProvider,
    pub server: WGServer,
    pub tunnel: ReverseTunnel,
    pub daemon: Arc<Mutex<Daemon>>,
    pub transmission_manager: Arc<Mutex<TransmissionManager>>,
    pub client_data: ClientData,
}

impl Context {
    pub async fn new(
        daemon: Arc<Mutex<Daemon>>,
        client_data: ClientData,
        server_data: ServerData,
    ) -> Result<Self, Error> {
        let token_provider = TokenProvider::new();

        let server = WGServer::new(server_data.grpc_addr);

        let tunnel = ReverseTunnel::new(server_data.tunn_addr);

        let dump_dir = DumpDir::new(*DISK_SIZE / 2).await;

        let transmission_manager = TransmissionManager::new(
            server.clone(),
            dump_dir,
            token_provider.clone(),
            server_data.grpc_addr.ip().to_string(),
            client_data.platform,
        );

        Ok(Self {
            token_provider,
            server,
            tunnel,
            daemon,
            client_data,
            transmission_manager: Arc::new(Mutex::new(transmission_manager)),
        })
    }
}
