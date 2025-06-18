use crate::arguments::Arguments;
use crate::constants::DISK_SIZE;
use crate::daemon::Daemon;
use crate::data_transmission::dump_dir::DumpDir;
use crate::data_transmission::transmission_manager::TransmissionManager;
use crate::platform::Platform;
use crate::reverse_tunnel::ReverseTunnel;
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
    pub transmission_manager: TransmissionManager,
}

impl Context {
    pub async fn new(
        arguments: Arguments,
        daemon: Arc<Mutex<Daemon>>,
        platform: Platform,
    ) -> Result<Self, Error> {
        let token_provider = TokenProvider::new();

        let server = WGServer::new(arguments.addr.clone(), arguments.port);

        let tunnel = ReverseTunnel::new(&arguments.tunnel_addr, arguments.tunnel_port).unwrap();

        let dump_dir = DumpDir::new(*DISK_SIZE / 2).await;

        let transmission_manager = TransmissionManager::new(
            server.clone(),
            dump_dir,
            token_provider.clone(),
            arguments.addr.clone(),
            platform,
        );

        Ok(Self {
            token_provider,
            server,
            tunnel,
            daemon,
            transmission_manager,
        })
    }
}
