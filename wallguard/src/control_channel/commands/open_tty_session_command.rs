use crate::context::Context;
use crate::control_channel::command::ExecutableCommand;
use crate::pty::{Pty, PtyReader, PtyWriter};
use crate::reverse_tunnel::{TunnelReader, TunnelWriter};
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use std::sync::Arc;
use wallguard_common::protobuf::wallguard_tunnel::client_frame::Message as ClientMessage;
use wallguard_common::protobuf::wallguard_tunnel::server_frame::Message as ServerMessage;
use wallguard_common::protobuf::wallguard_tunnel::{ClientFrame, DataFrame};

pub struct OpenTtySessionCommand {
    context: Context,
    token: String,
}

impl OpenTtySessionCommand {
    pub fn new(context: Context, token: String) -> Self {
        Self { context, token }
    }
}

impl ExecutableCommand for OpenTtySessionCommand {
    async fn execute(self) -> Result<(), nullnet_liberror::Error> {
        log::debug!("Received OpenTtySessionCommand");

        let pty = Pty::new("/bin/sh")?;

        let Ok(tunnel) = self.context.tunnel.request_channel(&self.token).await else {
            return Err("Cant establish tunnel connection").handle_err(location!());
        };

        tokio::spawn(async move {
            tokio::select! {
                _ = stream_to_pty(tunnel.reader, pty.writer) => {},
                _ = pty_to_stream(tunnel.writer, pty.reader) => {},
            }
        });

        Ok(())
    }
}

async fn stream_to_pty(reader: TunnelReader, writer: PtyWriter) -> Result<(), Error> {
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

        let writer = Arc::clone(&writer);
        tokio::task::spawn_blocking(move || writer.lock().unwrap().write_all(&data_frame.data))
            .await
            .handle_err(location!())?
            .handle_err(location!())?;
    }
}

async fn pty_to_stream(writer: TunnelWriter, reader: PtyReader) -> Result<(), Error> {
    loop {
        let reader = Arc::clone(&reader);

        let data = tokio::task::spawn_blocking(move || {
            let mut buf = [0u8; 8196];
            match reader.lock().unwrap().read(&mut buf) {
                Ok(0) => Ok(Vec::new()), // EOF
                Ok(n) => Ok(buf[..n].to_vec()),
                Err(err) => Err(err),
            }
        })
        .await
        .handle_err(location!())?
        .handle_err(location!())?;

        writer
            .lock()
            .await
            .send(ClientFrame {
                message: Some(ClientMessage::Data(DataFrame { data })),
            })
            .await
            .handle_err(location!())?;
    }
}
