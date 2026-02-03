use crate::control_channel::command::ExecutableCommand;
use crate::remote_desktop::RemoteDesktopManager;
use crate::{context::Context, reverse_tunnel::TunnelInstance};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::sync::mpsc;

pub struct OpenRemoteDesktopSessionCommand {
    context: Context,
    token: String,
}

impl OpenRemoteDesktopSessionCommand {
    pub fn new(context: Context, token: String) -> Self {
        Self { context, token }
    }
}

impl ExecutableCommand for OpenRemoteDesktopSessionCommand {
    async fn execute(self) -> Result<(), Error> {
        log::debug!("Received OpenRemoteDesktopSessionCommand");

        if !self
            .context
            .client_data
            .platform
            .can_open_remote_desktop_session()
        {
            return Err("Cannot open remote desktop session: unsupported session")
                .handle_err(location!());
        }

        let Ok(tunnel) = self.context.tunnel.request_channel(&self.token).await else {
            return Err("Cant establish tunnel connection").handle_err(location!());
        };

        let Some(mut rdm) = self.context.remote_desktop_manager.clone() else {
            return Err("Remote Desktop is not available").handle_err(location!());
        };

        tokio::spawn(async move {
            let (sender, receiver) = mpsc::channel(64);
            let id = rdm.on_client_connected(sender).await;

            let (reader, writer) = tokio::io::split(tunnel);

            tokio::select! {
                _ = stream_to_system(reader, rdm.clone(), id) => {},
                _ = system_to_stream(writer, receiver) => {},
            }

            let _ = rdm.on_client_disconnected(id).await;
        });

        Ok(())
    }
}

async fn stream_to_system(
    mut reader: ReadHalf<TunnelInstance>,
    remote_desktop_manager: RemoteDesktopManager,
    client_id: u128,
) -> Result<(), Error> {
    loop {
        let mut buffer = [0; 4096];
        let bytes = reader.read(&mut buffer).await.handle_err(location!())?;

        let message = buffer[..bytes].to_vec();

        if let Err(err) = remote_desktop_manager
            .on_client_message(client_id, message)
            .await
        {
            log::error!(
                "OpenRemoteDesktopSessionCommand: Failed to handle client message: {}",
                err.to_str()
            );
        }
    }
}

async fn system_to_stream(
    mut writer: WriteHalf<TunnelInstance>,
    mut receiver: mpsc::Receiver<Vec<u8>>,
) -> Result<(), Error> {
    while let Some(data) = receiver.recv().await {
        writer
            .write_all(data.as_slice())
            .await
            .handle_err(location!())?;
    }

    Ok(())
}
