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

#[cfg(not(target_os = "freebsd"))]
use crate::remote_desktop::RemoteDesktopManager;

#[derive(Clone, Debug)]
pub struct Context {
    pub token_provider: TokenProvider,
    pub server: WGServer,
    pub tunnel: ReverseTunnel,
    pub daemon: Arc<Mutex<Daemon>>,
    pub client_data: ClientData,
    pub(crate) transmission_manager: Arc<Mutex<TransmissionManager>>,

    #[cfg(not(target_os = "freebsd"))]
    pub remote_desktop_manager: Option<RemoteDesktopManager>,
}

impl Context {
    pub async fn new(
        daemon: Arc<Mutex<Daemon>>,
        client_data: ClientData,
        server_data: ServerData,
    ) -> Result<Self, Error> {
        let token_provider = TokenProvider::new();

        let server = WGServer::new(server_data.grpc_addr);

        // TODO
        let tunnel_acceptor_addr = format!("{}:{}", server_data.grpc_addr.ip(), 7777);
        let tunnel = ReverseTunnel::new(tunnel_acceptor_addr.parse().unwrap());

        let dump_dir = DumpDir::new(*DISK_SIZE / 2).await;

        // Initialise RemoteDesktopManager before TransmissionManager so we can
        // pass the *actual* rd_available flag (based on whether enigo connected)
        // rather than just checking for an X11 socket that may not be usable.
        #[cfg(not(target_os = "freebsd"))]
        let remote_desktop_manager = if client_data.platform.can_open_remote_desktop_session() {
            match RemoteDesktopManager::new() {
                Ok(rdm) => Some(rdm),
                Err(err) => {
                    log::warn!(
                        "Remote Desktop unavailable (display not accessible): {}",
                        err.to_str()
                    );
                    None
                }
            }
        } else {
            None
        };

        // On FreeBSD there is no remote_desktop_manager at all.
        let rd_available = {
            #[cfg(not(target_os = "freebsd"))]
            { remote_desktop_manager.is_some() }
            #[cfg(target_os = "freebsd")]
            { false }
        };

        let transmission_manager = TransmissionManager::new(
            server.clone(),
            dump_dir,
            token_provider.clone(),
            server_data.grpc_addr.ip().to_string(),
            client_data.platform,
            rd_available,
        );

        Ok(Self {
            token_provider,
            server,
            tunnel,
            daemon,
            client_data,
            transmission_manager: Arc::new(Mutex::new(transmission_manager)),

            #[cfg(not(target_os = "freebsd"))]
            remote_desktop_manager,
        })
    }
}
