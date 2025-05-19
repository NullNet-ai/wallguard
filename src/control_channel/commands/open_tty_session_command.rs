use std::sync::Arc;

use nullnet_liberror::{location, ErrorHandler, Location};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;

use crate::app_context::AppContext;
use crate::control_channel::command::ExecutableCommand;
use crate::pty::{Pty, PtyReader, PtyWriter};

pub struct OpenTtySessionCommand {
    context: AppContext,
    token: String,
}

impl OpenTtySessionCommand {
    pub fn new(context: AppContext, token: String) -> Self {
        Self { context, token }
    }
}

impl ExecutableCommand for OpenTtySessionCommand {
    async fn execute(self) -> Result<(), nullnet_liberror::Error> {
        log::debug!("Received OpenTtySessionCommand");

        let pty = Pty::new("/bin/sh")?;

        let Ok(stream) = self.context.tunnel.request_channel(&self.token).await else {
            return Err("Cant establish tunnel connection").handle_err(location!());
        };

        let (reader, writer) = tokio::io::split(stream);

        tokio::spawn(async move {
            tokio::select! {
                _ = stream_to_pty(reader, pty.writer) => {},
                _ = pty_to_stream(writer, pty.reader) => {},
            }
        });

        Ok(())
    }
}

async fn stream_to_pty(mut reader: ReadHalf<TcpStream>, writer: PtyWriter) {
    let mut buf = [0u8; 8196];

    loop {
        match reader.read(&mut buf).await {
            Ok(0) => {
                log::debug!("Stream EOF");
                break;
            }
            Ok(n) => {
                let chunk = buf[..n].to_vec();
                let writer = Arc::clone(&writer);
                if let Err(err) =
                    tokio::task::spawn_blocking(move || writer.lock().unwrap().write_all(&chunk))
                        .await
                        .unwrap_or_else(|e| Err(std::io::Error::new(std::io::ErrorKind::Other, e)))
                {
                    log::error!("Error writing to PTY: {}", err);
                    break;
                }
            }
            Err(err) => {
                log::error!("Error reading from stream: {}", err);
                break;
            }
        }
    }
}

async fn pty_to_stream(mut writer: WriteHalf<TcpStream>, reader: PtyReader) {
    loop {
        let reader = Arc::clone(&reader);
        let result = tokio::task::spawn_blocking(move || {
            let mut buf = [0u8; 8196];
            match reader.lock().unwrap().read(&mut buf) {
                Ok(0) => Ok(Vec::new()), // EOF
                Ok(n) => Ok(buf[..n].to_vec()),
                Err(err) => Err(err),
            }
        })
        .await
        .unwrap_or_else(|e| Err(std::io::Error::new(std::io::ErrorKind::Other, e)));

        match result {
            Ok(data) if data.is_empty() => {
                log::info!("PTY EOF");
                break;
            }
            Ok(data) => {
                if let Err(err) = writer.write_all(&data).await {
                    log::error!("Failed to write to stream: {}", err);
                    break;
                }
            }
            Err(err) => {
                log::error!("Failed to read from PTY: {}", err);
                break;
            }
        }
    }
}
