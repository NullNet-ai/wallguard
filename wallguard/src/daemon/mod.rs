#[rustfmt::skip]
mod wallguard_cli;
mod cli_server;
mod state;

use crate::arguments::Arguments;
use crate::context::Context;
use crate::control_channel::ControlChannel;
use crate::daemon::cli_server::CliServer;
use crate::daemon::state::DaemonState;
use crate::platform::Platform;
use crate::storage::{Secret, Storage};
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use wallguard_cli::wallguard_cli_server::WallguardCliServer;
use wallguard_cli::Status;

#[derive(Debug, Default)]
pub struct Daemon {
    uuid: String,
    arguments: Arguments,
    state: DaemonState,
    platform: Platform,
}

impl Daemon {
    pub async fn run(
        uuid: impl Into<String>,
        arguments: Arguments,
        platform: Platform,
    ) -> Result<(), Error> {
        let daemon = Arc::new(Mutex::new(Daemon {
            uuid: uuid.into(),
            arguments,
            state: DaemonState::default(),
            platform,
        }));

        if let Some(org_id) = Storage::get_value(Secret::OrgId).await {
            log::info!("Found org id {org_id}, attempting to connect");
            let _ = Daemon::join_org(daemon.clone(), org_id).await;
        } else {
            log::info!("No org ID, entering idle state");
        }

        let addr: SocketAddr = "127.0.0.1:54056".parse().unwrap();
        let cli_server = WallguardCliServer::new(CliServer::from(daemon));

        tonic::transport::Server::builder()
            .add_service(cli_server)
            .serve(addr)
            .await
            .handle_err(location!())
    }

    pub(crate) fn get_status(&self) -> Status {
        self.state.clone().into()
    }

    pub(crate) fn get_platform(&self) -> Platform {
        self.platform
    }

    pub(crate) async fn join_org(this: Arc<Mutex<Daemon>>, org_id: String) -> Result<(), String> {
        let mut lock = this.lock().await;
        match &lock.state {
            DaemonState::Idle => {
                Storage::set_value(Secret::OrgId, &org_id)
                    .await
                    .map_err(|err| err.to_str().to_string())?;

                let context = Context::new(lock.arguments.clone(), this.clone())
                    .await
                    .map_err(|err| err.to_str().to_string())?;

                let control_channel = ControlChannel::new(context, lock.uuid.clone(), org_id);

                lock.state = DaemonState::Connected(control_channel);

                Ok(())
            }
            _ => Err(format!(
                "Can not join a new organization from the current state: {}",
                lock.state
            )),
        }
    }

    pub(crate) async fn leave_org(this: Arc<Mutex<Daemon>>) -> Result<(), String> {
        let mut this = this.lock().await;

        match &this.state {
            DaemonState::Connected(control_channel) => {
                Storage::delete_value(Secret::OrgId)
                    .await
                    .map_err(|err| err.to_str().to_string())?;

                control_channel.terminate();

                this.state = DaemonState::Idle;
                Ok(())
            }
            DaemonState::Error(_) => {
                this.state = DaemonState::Idle;
                Ok(())
            }
            _ => Err(format!(
                "Can not leave current organization from the current state: {}",
                this.state
            )),
        }
    }

    pub(crate) async fn on_error(this: Arc<Mutex<Daemon>>, reason: impl Into<String>) {
        match this.lock().await.state.clone() {
            DaemonState::Connected(control_channel) => control_channel.terminate(),
            _ => {}
        };
        this.lock().await.state = DaemonState::Error(reason.into());
    }
}
