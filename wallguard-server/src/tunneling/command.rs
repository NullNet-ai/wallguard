use crate::{app_context::AppContext, reverse_tunnel::TunnelInstance};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::time::Duration;

const DEFAULT_TIMEOUT: Duration = Duration::from_millis(1_000);

#[derive(Debug, Clone)]
enum TunnelType {
    Ssh((String, String)),
    Tty,
    // Local Addr, Local Port, Protocol
    UI((String, u32, String)),
    _RemoteDesktop,
}

pub async fn establish_tunneled_ssh(
    context: &AppContext,
    device_id: &str,
    public_key: &str,
    username: &str,
) -> Result<TunnelInstance, Error> {
    let instance_id = context
        .orchestractor
        .get_any_client_instance(device_id)
        .await
        .ok_or("device not found")
        .handle_err(location!())?
        .lock()
        .await
        .instance_id
        .clone();

    establish_tunneled_channel(
        context,
        device_id,
        &instance_id,
        TunnelType::Ssh((public_key.into(), username.into())),
    )
    .await
}

pub async fn establish_tunneled_tty(
    context: &AppContext,
    device_id: &str,
) -> Result<TunnelInstance, Error> {
    let instance_id = context
        .orchestractor
        .get_any_client_instance(device_id)
        .await
        .ok_or("device not found")
        .handle_err(location!())?
        .lock()
        .await
        .instance_id
        .clone();

    establish_tunneled_channel(context, device_id, &instance_id, TunnelType::Tty).await
}

pub async fn _establish_tunneled_rd(
    context: &AppContext,
    device_id: &str,
) -> Result<TunnelInstance, Error> {
    let instance_id = context
        .orchestractor
        .get_any_client_instance(device_id)
        .await
        .ok_or("device not found")
        .handle_err(location!())?
        .lock()
        .await
        .instance_id
        .clone();

    establish_tunneled_channel(context, device_id, &instance_id, TunnelType::_RemoteDesktop).await
}

pub async fn establish_tunneled_ui(
    context: &AppContext,
    device_id: &str,
    protocol: &str,
    local_addr: &str,
    local_port: u32,
) -> Result<TunnelInstance, Error> {
    let instance_id = context
        .orchestractor
        .get_any_client_instance(device_id)
        .await
        .ok_or("device not found")
        .handle_err(location!())?
        .lock()
        .await
        .instance_id
        .clone();

    establish_tunneled_channel(
        context,
        device_id,
        &instance_id,
        TunnelType::UI((local_addr.into(), local_port, protocol.into())),
    )
    .await
}

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
        TunnelType::_RemoteDesktop => client.request_remote_desktop_session(token.clone()).await?,
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
