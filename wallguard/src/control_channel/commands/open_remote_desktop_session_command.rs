use crate::control_channel::command::ExecutableCommand;
use crate::remote_desktop::RemoteDesktopManager;
use crate::reverse_tunnel::TunnelWriter;
use crate::{context::Context, reverse_tunnel::TunnelReader};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use tokio::sync::mpsc;
use wallguard_common::protobuf::wallguard_tunnel::client_frame::Message as ClientMessage;
use wallguard_common::protobuf::wallguard_tunnel::server_frame::Message as ServerMessage;
use wallguard_common::protobuf::wallguard_tunnel::{ClientFrame, DataFrame};

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

            tokio::select! {
                _ = stream_to_system(tunnel.reader, rdm.clone(), id) => {},
                _ = system_to_stream(tunnel.writer, receiver) => {},
            }

            let _ = rdm.on_client_disconnected(id).await;
        });

        Ok(())
    }
}

async fn stream_to_system(
    reader: TunnelReader,
    remote_desktop_manager: RemoteDesktopManager,
    client_id: u128,
) -> Result<(), Error> {
    loop {
        let message = reader
            .lock()
            .await
            .message()
            .await
            .handle_err(location!())?
            .ok_or("End of stream")
            .handle_err(location!())?
            .message
            .ok_or("Unexpected empty message")
            .handle_err(location!())?;

        let ServerMessage::Data(data_frame) = message else {
            return Err("Unexpected message type").handle_err(location!())?;
        };

        if let Err(err) = remote_desktop_manager
            .on_client_message(client_id, data_frame.data)
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
    writer: TunnelWriter,
    mut receiver: mpsc::Receiver<Vec<u8>>,
) -> Result<(), Error> {
    while let Some(data) = receiver.recv().await {
        writer
            .lock()
            .await
            .send(ClientFrame {
                message: Some(ClientMessage::Data(DataFrame { data })),
            })
            .await
            .handle_err(location!())?;
    }

    Ok(())
}
