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

        // Create a fresh RemoteDesktopManager for this session.  Doing this
        // on-demand (rather than caching it in Context) means:
        //   • The agent never panics at startup when no display is available.
        //   • The first session after a user logs in just works — no restart
        //     needed.
        let mut rdm = RemoteDesktopManager::new().map_err(|err| {
            log::warn!("Cannot open remote desktop session: {}", err.to_str());
            err
        })?;

        let Ok(tunnel) = self.context.tunnel.request_channel(&self.token).await else {
            return Err("Cant establish tunnel connection").handle_err(location!());
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
        // The server prefixes every input event with a 4-byte LE length so we
        // always read exactly one complete JSON message, never a partial one.
        let mut len_buf = [0u8; 4];
        if reader.read_exact(&mut len_buf).await.is_err() {
            break;
        }

        let len = u32::from_le_bytes(len_buf) as usize;

        let mut message = vec![0u8; len];
        if reader.read_exact(&mut message).await.is_err() {
            break;
        }

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

    Ok(())
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
