use crate::rtty::TTYServer;
use nullnet_libconfmon::Platform;
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use nullnet_libtunnel::{Client, Config};
use std::net::SocketAddr;
use tokio::sync::broadcast;

pub struct RemoteAccessSession {
    shutdown_tx: broadcast::Sender<()>,
}

impl RemoteAccessSession {
    pub fn tty(tunnel_id: String, server_addr: SocketAddr, platform: Platform) -> Self {
        let (tx, _) = broadcast::channel(8);

        let rtty_server_addr = "127.0.0.1:3030".parse().unwrap();

        let rtty = TTYServer::new(rtty_server_addr, platform);

        let tunnel = Client::new(Config {
            id: tunnel_id,
            server_addr,
            local_addr: rtty_server_addr,
            reconnect_timeout: None,
            heartbeat_timeout: None,
        });

        tokio::spawn(Self::run_tty_server(rtty, tx.subscribe()));
        tokio::spawn(Self::run_tunnel_client(tunnel, tx.subscribe()));

        Self { shutdown_tx: tx }
    }

    pub fn ui(
        tunnel_id: String,
        protocol: &str,
        server_addr: SocketAddr,
        _: Platform,
    ) -> Result<Self, Error> {
        let (tx, _) = broadcast::channel(8);

        let local_addr = match protocol.to_lowercase().as_str() {
            "http" => "127.0.0.1:80".parse().unwrap(),
            "https" => "127.0.0.1:443".parse().unwrap(),
            _ => Err("Unsupported protocol").handle_err(location!())?,
        };

        let tunnel = Client::new(Config {
            id: tunnel_id,
            server_addr,
            local_addr,
            reconnect_timeout: None,
            heartbeat_timeout: None,
        });

        tokio::spawn(Self::run_tunnel_client(tunnel, tx.subscribe()));

        Ok(Self { shutdown_tx: tx })
    }

    pub fn terminate(&self) {
        let _ = self.shutdown_tx.send(());
    }

    async fn run_tty_server(mut server: TTYServer, mut receiver: broadcast::Receiver<()>) {
        if let Err(err) = server.start().await {
            log::error!("Failed to run TTY server: {err:?}");
        };

        match receiver.recv().await {
            Ok(_) => server.stop().await,
            Err(err) => log::error!("Failed to receive termination signal: {err:?}"),
        };
    }

    async fn run_tunnel_client(mut client: Client, mut receiver: broadcast::Receiver<()>) {
        tokio::select! {
            _ = client.run() => {},
            _ = receiver.recv() => {}
        }
    }
}
