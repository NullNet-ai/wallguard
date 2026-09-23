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
    pub client_data: ClientData,
    pub(crate) transmission_manager: Arc<Mutex<TransmissionManager>>,
}

impl Context {
    pub async fn new(
        daemon: Arc<Mutex<Daemon>>,
        client_data: ClientData,
        server_data: ServerData,
        batch_size: usize,
    ) -> Result<Self, Error> {
        let token_provider = TokenProvider::new();

        let server = WGServer::new(server_data.grpc_addr);

        // TODO
        let tunnel_acceptor_addr = format!("{}:{}", server_data.grpc_addr.ip(), 7777);
        let tunnel = ReverseTunnel::new(tunnel_acceptor_addr.parse().unwrap());

        let dump_dir = DumpDir::new(*DISK_SIZE / 2).await;

        let mut transmission_manager = TransmissionManager::new(
            server.clone(),
            dump_dir,
            token_provider.clone(),
            server_data.grpc_addr.ip().to_string(),
            client_data.platform,
            batch_size,
        );
        transmission_manager.start_retransmission_handler();

        Ok(Self {
            token_provider,
            server,
            tunnel,
            daemon,
            client_data,
            transmission_manager: Arc::new(Mutex::new(transmission_manager)),
        })
    }

    /// Stops every background task owned by this context and drops its gRPC
    /// channel. Must be called before the context is replaced or abandoned:
    /// the tasks hold clones of `server`, so otherwise they keep the old
    /// connection open indefinitely.
    pub async fn teardown(&self) {
        let mut manager = self.transmission_manager.lock().await;

        manager.terminate_packet_capture();
        manager.terminate_resource_monitoring();
        manager.terminate_sysconfig_monitoring();
        manager.terminate_services_monitoring();
        manager.terminate_retransmission_handler();

        drop(manager);

        self.server.reset().await;
    }
}
