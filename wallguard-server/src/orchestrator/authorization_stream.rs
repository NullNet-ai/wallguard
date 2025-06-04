use crate::protocol::wallguard_authorization::AuthorizationStatus;
use crate::protocol::wallguard_authorization::authorization_status::State;
use crate::{orchestrator::Orchestrator, protocol::wallguard_authorization::AuthorizationApproved};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tonic::Status;

const DEFAULT_HEALTHCHECK_TIME: Duration = Duration::from_secs(15);

pub(crate) type AuthorizationStream = mpsc::Sender<Result<AuthorizationStatus, Status>>;

#[derive(Debug, Clone)]
pub(crate) struct PendingAuth {
    device_uuid: String,
    complete: broadcast::Sender<()>,
    stream: AuthorizationStream,
}

impl PendingAuth {
    pub fn new(
        device_uuid: impl Into<String>,
        stream: AuthorizationStream,
        orchestrator: Orchestrator,
    ) -> Self {
        let (complete, _) = broadcast::channel(1);
        let device_uuid = device_uuid.into();

        tokio::spawn(healthcheck(
            stream.clone(),
            complete.subscribe(),
            orchestrator,
            device_uuid.clone(),
        ));

        Self {
            stream,
            device_uuid,
            complete,
        }
    }

    pub async fn send_approve(
        &self,
        app_id: impl Into<String>,
        app_secret: impl Into<String>,
    ) -> Result<(), Error> {
        let _ = self.complete.send(());

        let status = AuthorizationStatus {
            state: Some(State::Approved(AuthorizationApproved {
                app_id: Some(app_id.into()),
                app_secret: Some(app_secret.into()),
            })),
        };

        self.stream.send(Ok(status)).await.handle_err(location!())
    }

    pub async fn send_reject(&self) -> Result<(), Error> {
        let _ = self.complete.send(());

        let status = AuthorizationStatus {
            state: Some(State::Rejected(())),
        };

        self.stream.send(Ok(status)).await.handle_err(location!())
    }
}

async fn healthcheck(
    stream: AuthorizationStream,
    mut receiver: broadcast::Receiver<()>,
    orchestrator: Orchestrator,
    device_uuid: String,
) {
    tokio::select! {
        _ = receiver.recv() => {}
        _ = async move {
            loop {
                tokio::time::sleep(DEFAULT_HEALTHCHECK_TIME).await;

                let status = AuthorizationStatus {
                    state: Some(State::Pending(())),
                };

                if stream.send(Ok(status)).await.is_err() {
                    break;
                }
            }
        } => {}
    };

    let _ = orchestrator
        .on_client_authorization_completed(&device_uuid)
        .await;
}
