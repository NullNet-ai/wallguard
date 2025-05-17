use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::time::Duration;
use tokio::sync::mpsc;

use super::client::ControlStream;
use crate::protocol::wallguard_commands::WallGuardCommand;
use crate::protocol::wallguard_commands::wall_guard_command::Command;
use crate::token_provider::TokenProvider;

const DEFAULT_HEARTBEAT_TIME: Duration = Duration::from_secs(20);
const DEFAULT_TOKEN_UPDATE_TIME: Duration = Duration::from_secs(60);

/// Handles the control stream for a connected client by concurrently managing:
/// - periodic health checks (heartbeat messages),
/// - token authentication updates,
/// - and listening for a shutdown signal to terminate the task.
///
/// # Parameters
/// - `device_id`: Unique identifier of the connected device.
/// - `stream`: The control stream for communication with the client.
/// - `token_provider`: Provides authentication tokens for the client.
/// - `complete`: An `mpsc::Sender<String>` channel used to notify that the control stream
///   task for the given `device_id` has completed.
pub(crate) async fn control_stream_task(
    device_id: String,
    stream: ControlStream,
    token_provider: TokenProvider,
    complete: mpsc::Sender<String>,
) {
    tokio::select! {
        hres = healthcheck(stream.clone(), token_provider.clone()) => {
            if let Err(err) = hres {
                log::error!(
                    "Health check for client with device ID '{}' failed: {}",
                    device_id,
                    err.to_str()
                );
            }
        },
        ares = authstream(stream, token_provider) => {
            if let Err(err) = ares {
                log::error!(
                    "Authentication stream for client with device ID '{}' failed: {}",
                    device_id,
                    err.to_str()
                );
            }
        },
    };

    let _ = complete.send(device_id).await.handle_err(location!());
}

/// Periodically sends heartbeat messages to the connected client to confirm
/// the connection remains active and healthy.
///
/// # Parameters
/// - `stream`: The control stream to which heartbeat messages are sent.
/// - `token_provider`: Token provider instance (currently unused in heartbeat,
///   but may be used in future for authenticated heartbeat messages).
///
/// # Errors
/// Returns an error if sending a heartbeat message fails.
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

/// Periodically sends updated authentication tokens to the client to maintain
/// an authenticated session.
///
/// # Parameters
/// - `stream`: The control stream to which updated token commands are sent.
/// - `token_provider`: Responsible for fetching fresh authentication tokens.
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
