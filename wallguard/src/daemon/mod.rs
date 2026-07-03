#[rustfmt::skip]
mod cli_server;
mod state;

use crate::client_data::ClientData;
use crate::context::Context;
use crate::control_channel::ControlChannel;
use crate::daemon::cli_server::CliServer;
use crate::daemon::state::DaemonState;
use crate::server_data::ServerData;
use crate::storage::{Secret, Storage};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use wallguard_common::protobuf::wallguard_cli::Status;
use wallguard_common::protobuf::wallguard_cli::wallguard_cli_server::WallguardCliServer;

#[derive(Debug)]
pub struct Daemon {
    client_data: ClientData,
    server_data: ServerData,
    state: DaemonState,
    connect_handle: Option<tokio::task::JoinHandle<()>>,
    batch_size: usize,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl Daemon {
    pub async fn run(client_data: ClientData, server_data: ServerData) -> Result<(), Error> {
        let batch_size = server_data.batch_size;
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let daemon = Arc::new(Mutex::new(Daemon {
            client_data,
            server_data,
            state: DaemonState::default(),
            connect_handle: None,
            batch_size,
            shutdown_tx: Some(shutdown_tx),
        }));

        if let Some(code) = Storage::get_value(Secret::InstallationCode).await {
            log::info!("Found installation code {code}, attempting to connect");
            let _ = Daemon::join_org(daemon.clone(), code).await;
        } else {
            log::info!("No org ID, entering idle state");
        }

        log::info!("Starting CLI server");

        let addr: SocketAddr = "127.0.0.1:54056".parse().unwrap();
        let cli_server = WallguardCliServer::new(CliServer::from(daemon));

        tonic::transport::Server::builder()
            .add_service(cli_server)
            .serve_with_shutdown(addr, async {
                let _ = shutdown_rx.await;
                log::info!("Shutdown requested, stopping CLI server");
            })
            .await
            .handle_err(location!())
    }

    pub(crate) async fn get_status(&self) -> Status {
        self.state.clone().into_status().await
    }

    pub(crate) async fn join_org(
        this: Arc<Mutex<Daemon>>,
        installation_code: String,
    ) -> Result<(), String> {
        let mut lock = this.lock().await;

        if matches!(lock.state, DaemonState::Connecting) {
            return Err(
                "Already connecting to an organization. Run `leave` to cancel first.".into(),
            );
        }
        if matches!(lock.state, DaemonState::Connected(_)) {
            return Err(
                "Already connected to an organization. Run `leave` to disconnect first.".into(),
            );
        }

        Storage::set_value(Secret::InstallationCode, &installation_code)
            .await
            .map_err(|err| err.to_str().to_string())?;

        let context = Context::new(
            this.clone(),
            lock.client_data.clone(),
            lock.server_data.clone(),
            lock.batch_size,
        )
        .await
        .map_err(|err| err.to_str().to_string())?;

        // Set state and store the task handle atomically so leave_org can abort it.
        lock.state = DaemonState::Connecting;
        let handle = tokio::spawn(async move { Daemon::connect(context, 0).await });
        lock.connect_handle = Some(handle);

        Ok(())
    }

    /// Retries connecting after the agent gave up following its bounded
    /// retries (see `DaemonState::Error`). Only valid from `Error`; use
    /// `join_org` to connect for the first time and `leave_org` to cancel
    /// an in-progress or established connection.
    pub(crate) async fn reconnect(this: Arc<Mutex<Daemon>>) -> Result<(), String> {
        let mut lock = this.lock().await;

        if !matches!(lock.state, DaemonState::Error(_)) {
            return Err("Not in an error state; nothing to reconnect.".into());
        }

        if Storage::get_value(Secret::InstallationCode).await.is_none() {
            return Err("No installation code found.".into());
        }

        let context = Context::new(
            this.clone(),
            lock.client_data.clone(),
            lock.server_data.clone(),
            lock.batch_size,
        )
        .await
        .map_err(|err| err.to_str().to_string())?;

        lock.state = DaemonState::Connecting;
        let handle = tokio::spawn(async move { Daemon::connect(context, 0).await });
        lock.connect_handle = Some(handle);

        Ok(())
    }

    pub(crate) async fn leave_org(this: Arc<Mutex<Daemon>>) -> Result<(), String> {
        let mut this = this.lock().await;

        match &this.state {
            DaemonState::Idle => Err("Not connected to any organization.".into()),

            DaemonState::Connecting => {
                if let Some(handle) = this.connect_handle.take() {
                    handle.abort();
                }
                let _ = Storage::delete_value(Secret::InstallationCode).await;
                this.state = DaemonState::Idle;
                Ok(())
            }

            DaemonState::Connected(control_channel) => {
                Storage::delete_value(Secret::InstallationCode)
                    .await
                    .map_err(|err| err.to_str().to_string())?;

                control_channel.terminate().await;

                this.state = DaemonState::Idle;
                Ok(())
            }

            DaemonState::Error(_) => {
                let _ = Storage::delete_value(Secret::InstallationCode).await;
                this.state = DaemonState::Idle;
                Ok(())
            }
        }
    }

    /// Gracefully tears down any active connection and stops the CLI gRPC
    /// server, allowing `main` to exit and release the single-instance lock.
    /// Unlike `leave_org`, the stored installation code is kept so the agent
    /// automatically rejoins its organization the next time it starts.
    pub(crate) async fn shutdown(this: Arc<Mutex<Daemon>>) -> Result<(), String> {
        let mut lock = this.lock().await;

        match &lock.state {
            DaemonState::Connecting => {
                if let Some(handle) = lock.connect_handle.take() {
                    handle.abort();
                }
            }
            DaemonState::Connected(control_channel) => {
                control_channel.terminate().await;
            }
            DaemonState::Idle | DaemonState::Error(_) => {}
        }

        lock.state = DaemonState::Idle;

        match lock.shutdown_tx.take() {
            Some(tx) => {
                let _ = tx.send(());
                Ok(())
            }
            None => Err("Shutdown already in progress.".into()),
        }
    }

    pub(crate) async fn on_error(this: Arc<Mutex<Daemon>>, reason: impl Into<String>) {
        if let DaemonState::Connected(control_channel) = this.lock().await.state.clone() {
            control_channel.terminate().await
        }

        this.lock().await.state = DaemonState::Error(reason.into());
    }

    pub(crate) async fn connect(context: Context, attempt: u32) {
        let daemon = context.clone().daemon;

        daemon.lock().await.state = DaemonState::Connecting;

        let backoff = reconnect_backoff(attempt);
        if !backoff.is_zero() {
            log::info!(
                "Reconnecting in {}s (attempt {})",
                backoff.as_secs(),
                attempt
            );
            tokio::time::sleep(backoff).await;
        }

        context.server.reset().await;

        if context.server.get_interface().await.is_err() {
            Daemon::on_error(daemon, "Failed to connect to the server").await;
            return;
        }

        if let Some(installation_code) = Storage::get_value(Secret::InstallationCode).await {
            let control_channel = ControlChannel::new(context, installation_code, attempt);
            daemon.lock().await.state = DaemonState::Connected(Box::new(control_channel));
        } else {
            Daemon::on_error(daemon, "Failed to obtain installation code").await;
        }
    }
}

fn reconnect_backoff(attempt: u32) -> std::time::Duration {
    const MAX: std::time::Duration = std::time::Duration::from_secs(60);
    if attempt == 0 {
        return std::time::Duration::ZERO;
    }
    std::time::Duration::from_secs(5u64.saturating_mul(1u64 << (attempt - 1).min(10))).min(MAX)
}
