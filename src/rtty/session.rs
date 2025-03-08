use futures_util::{SinkExt, StreamExt};
use nullnet_libconfmon::Platform;
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{channel, Sender};
use tokio::task::{self, JoinHandle};
use uuid::Uuid;
use warp::ws::WebSocket;

pub struct Session {
    /// Used to send a `shutdown` signal to the worker.
    shutdown_signal: Sender<()>,
    /// The worker's handle.
    handle: JoinHandle<()>,
    /// The unique identifier for the session.
    pub id: uuid::Uuid,
}

impl Session {
    pub fn spawn(websocket: WebSocket, platform: Platform, complete_signal: Sender<Uuid>) -> Self {
        let (shutdown_signal, mut shutdown_receiver) = channel(1);

        let id = Uuid::new_v4();

        let handle = tokio::spawn(async move {
            let id = id.clone();

            tokio::select! {
                result = Self::run(websocket, platform) => {
                    if let Err(err) = result {
                        log::error!("RemoteTTY session error: {}", err.to_str())
                    } else {
                        log::info!("RemoteTTY session completed")
                    }

                },
                _ = shutdown_receiver.recv() => log::warn!("RemoteTTY session cancelled")
            };

            let _ = complete_signal.send(id).await.handle_err(location!());
        });

        Self {
            id,
            handle,
            shutdown_signal,
        }
    }

    /// This function WILL NOT send complete signal
    pub async fn shutdown(self) -> Result<(), Error> {
        self.shutdown_signal
            .send(())
            .await
            .handle_err(location!())?;
        self.handle.await.handle_err(location!())
    }

    async fn run(websocket: WebSocket, platform: Platform) -> Result<(), Error> {
        let pty = NativePtySystem::default()
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_height: 0,
                pixel_width: 0,
            })
            .handle_err(location!())?;

        let _ = pty
            .slave
            .spawn_command(CommandBuilder::new(Self::command(platform)))
            .handle_err(location!())?;

        let reader = pty.master.try_clone_reader().handle_err(location!())?;
        let mut writer = pty.master.take_writer().handle_err(location!())?;

        let (mut ws_tx, mut ws_rx) = websocket.split();

        let ws_to_pty: JoinHandle<Result<(), Error>> = task::spawn(async move {
            while let Some(Ok(msg)) = ws_rx.next().await {
                writer.write_all(msg.as_bytes()).handle_err(location!())?;
            }

            Ok(())
        });

        let pty_to_ws: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
            let reader = Arc::new(Mutex::new(reader));

            loop {
                let reader = reader.clone();

                let read_result = tokio::task::spawn_blocking(move || -> Result<String, Error> {
                    let mut buffer = [0u8; 1024];
                    match reader.lock().handle_err(location!())?.read(&mut buffer) {
                        Ok(0) => Ok(String::new()),
                        Ok(n) => Ok(String::from_utf8_lossy(&buffer[..n]).to_string()),
                        Err(err) => Err(err).handle_err(location!()),
                    }
                })
                .await
                .handle_err(location!())?;

                if let Ok(message) = read_result {
                    if message.is_empty() {
                        break;
                    }

                    if let Err(_) = ws_tx.send(warp::ws::Message::text(message)).await {
                        break;
                    }
                } else {
                    break;
                }
            }

            Ok(())
        });

        let (result1, result2) = tokio::join!(pty_to_ws, ws_to_pty);
        result1.handle_err(location!())??;
        result2.handle_err(location!())??;

        Ok(())
    }

    fn command(platform: Platform) -> &'static str {
        match platform {
            Platform::PfSense => "/etc/rc.initial",
            Platform::OPNsense => "/bin/sh",
        }
    }
}
