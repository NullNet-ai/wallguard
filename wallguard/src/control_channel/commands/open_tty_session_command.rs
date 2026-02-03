use std::sync::Arc;

use nullnet_liberror::{Error, ErrorHandler, Location, location};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};

use crate::context::Context;
use crate::control_channel::command::ExecutableCommand;
use crate::pty::{Pty, PtyReader, PtyWriter};
use crate::reverse_tunnel::TunnelInstance;

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
            let (reader, writer) = tokio::io::split(tunnel);

            tokio::select! {
                _ = stream_to_pty(reader, pty.writer) => {},
                _ = pty_to_stream(writer, pty.reader) => {},
            }
        });

        Ok(())
    }
}

async fn stream_to_pty(
    mut reader: ReadHalf<TunnelInstance>,
    writer: PtyWriter,
) -> Result<(), Error> {
    loop {
        let mut buffer = [0; 4096];
        let bytes = reader.read(&mut buffer).await.handle_err(location!())?;

        let message = buffer[..bytes].to_vec();

        let writer = Arc::clone(&writer);
        tokio::task::spawn_blocking(move || writer.lock().unwrap().write_all(&message))
            .await
            .handle_err(location!())?
            .handle_err(location!())?;
    }
}

async fn pty_to_stream(
    mut writer: WriteHalf<TunnelInstance>,
    reader: PtyReader,
) -> Result<(), Error> {
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
            .write_all(data.as_slice())
            .await
            .handle_err(location!())?;
    }
}
