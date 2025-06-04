use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::time::Duration;
use tokio::sync::mpsc;
use tonic::Status;

use crate::orchestrator::Orchestrator;
use crate::protocol::wallguard_commands::WallGuardCommand;
use crate::protocol::wallguard_commands::wall_guard_command::Command;
use crate::token_provider::TokenProvider;

const DEFAULT_HEARTBEAT_TIME: Duration = Duration::from_secs(20);
const DEFAULT_TOKEN_UPDATE_TIME: Duration = Duration::from_secs(60);

pub(crate) type ControlStream = mpsc::Sender<Result<WallGuardCommand, Status>>;

pub(crate) async fn control_stream_task(
    device_uuid: String,
    stream: ControlStream,
    token_provider: TokenProvider,
    orchestrator: Orchestrator,
) {
    tokio::select! {
        hres = healthcheck(stream.clone(), token_provider.clone()) => {
            if let Err(err) = hres {
                log::error!(
                    "Health check for client with device UUID '{}' failed: {}",
                    device_uuid,
                    err.to_str()
                );
            }
        },
        ares = authstream(stream, token_provider) => {
            if let Err(err) = ares {
                log::error!(
                    "Authentication stream for client with device UUID '{}' failed: {}",
                    device_uuid,
                    err.to_str()
                );
            }
        },
    };

    let _ = orchestrator.on_client_disconnected(&device_uuid).await;
}

async fn healthcheck(stream: ControlStream, _token_provider: TokenProvider) -> Result<(), Error> {
    loop {
        // The order here is important: this task starts as soon as the client connects,
        // even before the client has received the initial gRPC response. While we *can*
        // send messages immediately (they'll be queued in the channel), there's no value
        // in sending a heartbeat right after the connection is established.
        tokio::time::sleep(DEFAULT_HEARTBEAT_TIME).await;

        let heartbeat = WallGuardCommand {
            command: Some(Command::HeartbeatCommand(())),
        };

        stream.send(Ok(heartbeat)).await.handle_err(location!())?;
    }
}

async fn authstream(stream: ControlStream, token_provider: TokenProvider) -> Result<(), Error> {
    loop {
        let token = token_provider.get().await?;

        let command = WallGuardCommand {
            command: Some(Command::UpdateTokenCommand(token.jwt.clone())),
        };

        stream.send(Ok(command)).await.handle_err(location!())?;

        tokio::time::sleep(DEFAULT_TOKEN_UPDATE_TIME).await;
    }
}
