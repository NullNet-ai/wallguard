use crate::{app_context::AppContext, reverse_tunnel::TunnelInstance};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::time::Duration;

/// Timeout for awaiting the reverse tunnel connection
const DEFAULT_TIMEOUT: Duration = Duration::from_millis(1_000);

/// Represents the type of tunnel we want to establish.
/// Variants may contain protocol-specific configuration.
#[derive(Debug, Clone)]
enum TunnelType {
    Ssh((String, String)),
    Tty,
    // Local Addr, Local Port, Protocol
    UI((String, u32, String)),
    RemoteDesktop,
}

/// Establishes a tunneled SSH connection for a device using its `SSHKeypair`.
///
/// # Arguments
/// - `context`: The application context with orchestrator and tunnel services
/// - `device_id`: The device ID
/// - `instance_id`: Instance ID
/// - `public_key`: The SSH public key used for authentication
pub async fn establish_tunneled_ssh(
    context: &AppContext,
    device_id: &str,
    instance_id: &str,
    public_key: &str,
    username: &str,
) -> Result<TunnelInstance, Error> {
    establish_tunneled_channel(
        context,
        device_id,
        instance_id,
        TunnelType::Ssh((public_key.into(), username.into())),
    )
    .await
}

/// Establishes a tunneled TTY (terminal) connection to the specified device.
///
/// # Arguments
/// - `context`: The application context
/// - `device_id`: The device ID
/// - `instance_id`: Instance ID
pub async fn establish_tunneled_tty(
    context: &AppContext,
    device_id: &str,
    instance_id: &str,
) -> Result<TunnelInstance, Error> {
    establish_tunneled_channel(context, device_id, instance_id, TunnelType::Tty).await
}

/// Establishes a tunneled remote desktop connection to the specified device.
///
/// # Arguments
/// - `context`: The application context
/// - `device_id`: The device ID
/// - `instance_id`: Instance ID
pub async fn establish_tunneled_rd(
    context: &AppContext,
    device_id: &str,
    instance_id: &str,
) -> Result<TunnelInstance, Error> {
    establish_tunneled_channel(context, device_id, instance_id, TunnelType::RemoteDesktop).await
}

/// Establishes a tunneled UI session using a given protocol string.
///
/// # Arguments
/// - `context`: The application context
/// - `device_id`: The device ID
/// - `instance_id`: Instance ID
/// - `protocol`: The UI protocol string (to be replaced with enum in future)
/// - `local_addr`: IP address of the local web server to connect to
/// - `local_port`: Port of the local web server to connect to
pub async fn establish_tunneled_ui(
    context: &AppContext,
    device_id: &str,
    instance_id: &str,
    protocol: &str,
    local_addr: &str,
    local_port: u32,
) -> Result<TunnelInstance, Error> {
    establish_tunneled_channel(
        context,
        device_id,
        instance_id,
        TunnelType::UI((local_addr.into(), local_port, protocol.into())),
    )
    .await
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
    device_id: &str,
    instance_id: &str,
    r#type: TunnelType,
) -> Result<TunnelInstance, Error> {
    let client = context
        .orchestractor
        .get_client(device_id, instance_id)
        .await
        .ok_or_else(|| format!("Client with device ID '{device_id}' is not connected"))
        .handle_err(location!())?;

    let client = client.lock().await;

    let (token, receiver) = context.tunnel.expect_connection().await;

    match r#type {
        TunnelType::Ssh((public_key, username)) => {
            client
                .request_ssh_session(token.clone(), public_key, username)
                .await?
        }
        TunnelType::Tty => client.request_tty_session(token.clone()).await?,
        TunnelType::RemoteDesktop => client.request_remote_desktop_session(token.clone()).await?,
        TunnelType::UI((addr, port, protocol)) => {
            client
                .request_ui_session(token.clone(), addr, port, protocol)
                .await?
        }
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
