use crate::app_context::AppContext;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::time::Duration;
use tokio::net::TcpStream;

/// Timeout for awaiting the reverse tunnel connection
const DEFAULT_TIMEOUT: Duration = Duration::from_millis(1_000);

/// Represents the type of tunnel we want to establish.
/// Variants may contain protocol-specific configuration.
#[derive(Debug, Clone)]
enum TunnelType {
    Ssh(String),
    Tty,
    UI(String),
}

/// Establishes a tunneled SSH connection for a device using its `SSHKeypair`.
///
/// # Arguments
/// - `context`: The application context with orchestrator and tunnel services
/// - `device_uuid`: The device UUID
/// - `public_key`: The SSH public key used for authentication
pub async fn establish_tunneled_ssh(
    context: &AppContext,
    device_uuid: &str,
    public_key: &str,
) -> Result<TcpStream, Error> {
    establish_tunneled_channel(context, device_uuid, TunnelType::Ssh(public_key.into())).await
}

/// Establishes a tunneled TTY (terminal) connection to the specified device.
///
/// # Arguments
/// - `context`: The application context
/// - `device_uuid`: The device UUID
pub async fn establish_tunneled_tty(
    context: &AppContext,
    device_uuid: &str,
) -> Result<TcpStream, Error> {
    establish_tunneled_channel(context, device_uuid, TunnelType::Tty).await
}

/// Establishes a tunneled UI session using a given protocol string.
///
/// # Arguments
/// - `context`: The application context
/// - `device_uuid`: The device UUID
/// - `protocol`: The UI protocol string (to be replaced with enum in future)
pub async fn establish_tunneled_ui(
    context: &AppContext,
    device_uuid: &str,
    protocol: &str,
) -> Result<TcpStream, Error> {
    establish_tunneled_channel(context, device_uuid, TunnelType::UI(protocol.into())).await
}

/// Core handler that establishes a tunneled channel of the given `TunnelType`.
///
/// It retrieves a reverse tunnel token, sends a tunnel request to the orchestrator client,
/// and awaits the resulting connection with a timeout.
///
/// # Errors
/// Returns an error if the client is not connected, request fails, or connection times out.
async fn establish_tunneled_channel(
    context: &AppContext,
    device_uuid: &str,
    r#type: TunnelType,
) -> Result<TcpStream, Error> {
    let client = context
        .orchestractor
        .get_client(device_uuid)
        .await
        .ok_or_else(|| format!("Client with device UUID '{device_uuid}' is not connected"))
        .handle_err(location!())?;

    let client = client.lock().await;

    let (token, receiver) = context.tunnel.expect_connection().await;

    match r#type {
        TunnelType::Ssh(public_key) => {
            client
                .request_ssh_session(token.clone(), public_key)
                .await?
        }
        TunnelType::Tty => client.request_tty_session(token.clone()).await?,
        TunnelType::UI(protocol) => client.request_ui_session(token.clone(), protocol).await?,
    };

    tokio::select! {
        stream = receiver => {
            stream.handle_err(location!())
        }
        _ = tokio::time::sleep(DEFAULT_TIMEOUT) => {
            context.tunnel.cancel_expectation(&token).await;
            Err("Timeout exceeded while waiting for tunneled stream").handle_err(location!())
        }
    }
}
