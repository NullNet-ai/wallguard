use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{broadcast, watch};
use tracing::{info, warn};

use crate::config::Config;
use crate::proto::cli::{
    agent_control_server::{AgentControl, AgentControlServer},
    GracefulRestartRequest, GracefulRestartResponse, StatusRequest, StatusResponse,
};
use crate::proto::control::MonitoringStatus;
use crate::state::DaemonState;

pub async fn run_cli_server(
    config:      Arc<Config>,
    state_rx:    watch::Receiver<DaemonState>,
    shutdown_tx: broadcast::Sender<()>,
) {
    use std::os::unix::fs::PermissionsExt;
    use tokio::net::UnixListener;
    use tokio_stream::wrappers::UnixListenerStream;

    let sock_path = Config::cli_socket_path();

    // Remove stale socket from a previous run.
    let _ = std::fs::remove_file(sock_path);
    if let Some(parent) = std::path::Path::new(sock_path).parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            warn!("could not create socket dir {}: {e}", parent.display());
            return;
        }
    }

    let listener = match UnixListener::bind(sock_path) {
        Ok(l)  => l,
        Err(e) => {
            warn!("could not bind CLI socket {sock_path}: {e}");
            return;
        }
    };

    // Restrict socket to root only.
    if let Err(e) = std::fs::set_permissions(
        sock_path,
        std::fs::Permissions::from_mode(0o600),
    ) {
        warn!("could not set socket permissions: {e}");
    }

    info!("CLI socket listening on {sock_path}");

    let svc = CliServer {
        state_rx,
        config,
        start_time:  Instant::now(),
        shutdown_tx,
    };

    let mut shutdown_rx = svc.shutdown_tx.subscribe();

    let router = tonic::transport::Server::builder()
        .add_service(AgentControlServer::new(svc));

    let incoming = UnixListenerStream::new(listener);

    tokio::select! {
        result = router.serve_with_incoming(incoming) => {
            if let Err(e) = result {
                warn!("CLI server error: {e}");
            }
        }
        _ = shutdown_rx.recv() => {
            info!("CLI server shutting down");
        }
    }
}

struct CliServer {
    state_rx:    watch::Receiver<DaemonState>,
    config:      Arc<Config>,
    start_time:  Instant,
    shutdown_tx: broadcast::Sender<()>,
}

#[tonic::async_trait]
impl AgentControl for CliServer {
    async fn status(
        &self,
        _req: tonic::Request<StatusRequest>,
    ) -> Result<tonic::Response<StatusResponse>, tonic::Status> {
        let state = self.state_rx.borrow().clone();
        Ok(tonic::Response::new(StatusResponse {
            state:             state.to_proto_i32(),
            device_id:         self.config.device.id.clone(),
            server_url:        self.config.grpc_endpoint(),
            agent_version:     env!("CARGO_PKG_VERSION").to_string(),
            monitoring_status: Some(MonitoringStatus::default()),
            uptime_secs:       self.start_time.elapsed().as_secs(),
        }))
    }

    async fn graceful_restart(
        &self,
        req: tonic::Request<GracefulRestartRequest>,
    ) -> Result<tonic::Response<GracefulRestartResponse>, tonic::Status> {
        let timeout_ms   = req.into_inner().drain_timeout_ms;
        let shutdown_tx  = self.shutdown_tx.clone();
        info!(timeout_ms, "graceful restart requested — draining in-flight commands");
        tokio::spawn(async move {
            crate::lifecycle::upgrade::drain(timeout_ms).await;
            let _ = shutdown_tx.send(());
        });
        Ok(tonic::Response::new(GracefulRestartResponse {
            accepted: true,
            message:  "draining and shutting down".to_string(),
        }))
    }
}
