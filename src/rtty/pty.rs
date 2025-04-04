use nullnet_libconfmon::Platform;
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex as TokioMutex;
use tokio::sync::{broadcast, mpsc, oneshot};

const MAX_BUFFER_SIZE: usize = 10 * 1024;
const CHANNEL_CAPACITY: usize = 128;
const CLEAR_MESSAGE: &[u8] = "\x1B[H\x1B[2J".as_bytes();

type PtyReader = Box<dyn Read + Send>;
type PtyWriter = Box<dyn Write + Send>;

#[derive(Debug)]
pub struct Pty {
    buffer: Arc<TokioMutex<VecDeque<u8>>>,
    to_pty: mpsc::Sender<Vec<u8>>,
    from_pty: broadcast::Sender<Vec<u8>>,
    shutdown: oneshot::Sender<()>,
}

impl Pty {
    pub fn new(platform: Platform) -> Result<Self, Error> {
        let buffer = Arc::new(TokioMutex::new(VecDeque::new()));

        let (to_pty_tx, to_pty_rx) = mpsc::channel(CHANNEL_CAPACITY);
        let (from_pty_tx, _) = broadcast::channel(CHANNEL_CAPACITY);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        tokio::spawn(pty_routine(
            platform,
            buffer.clone(),
            to_pty_rx,
            from_pty_tx.clone(),
            shutdown_rx,
        ));

        Ok(Self {
            buffer,
            to_pty: to_pty_tx,
            from_pty: from_pty_tx,
            shutdown: shutdown_tx,
        })
    }

    pub async fn current_buffer(&self) -> Vec<u8> {
        self.buffer.lock().await.iter().copied().collect()
    }

    pub async fn subscribe(&self) -> broadcast::Receiver<Vec<u8>> {
        self.from_pty.subscribe()
    }

    pub async fn send(&self, message: Vec<u8>) -> Result<(), Error> {
        self.to_pty.send(message).await.handle_err(location!())
    }

    pub async fn shutdown(self) {
        let _ = self.shutdown.send(());
    }
}

fn pty_program(platform: &Platform) -> &'static str {
    match platform {
        Platform::PfSense => "/etc/rc.initial",
        Platform::OPNsense => "/bin/sh",
    }
}

async fn pty_routine(
    platform: Platform,
    buffer: Arc<TokioMutex<VecDeque<u8>>>,
    to_pty: mpsc::Receiver<Vec<u8>>,
    from_pty: broadcast::Sender<Vec<u8>>,
    mut shutdown: oneshot::Receiver<()>,
) -> Result<(), Error> {
    let to_pty = Arc::new(TokioMutex::new(to_pty));

    loop {
        let pty = NativePtySystem::default()
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_height: 0,
                pixel_width: 0,
            })
            .handle_err(location!())?;

        let mut child = pty
            .slave
            .spawn_command(CommandBuilder::new(pty_program(&platform)))
            .handle_err(location!())?;

        let reader = pty.master.try_clone_reader().handle_err(location!())?;
        let writer = pty.master.take_writer().handle_err(location!())?;

        tokio::select! {
            _ = reader_routine(reader, buffer.clone(), from_pty.clone()) => {
                log::debug!("Pty: reader terminated");
                break;
            },
            _ = writer_routine(writer, to_pty.clone()) => {
                log::debug!("Pty: writer terminated");
                break;
            },
            _ = tokio::task::spawn_blocking(move || child.wait()) => {
                log::debug!("Pty: command terminated, respawning ...");
                let clear = Vec::from(CLEAR_MESSAGE);
                let _ = from_pty.send(clear);
                buffer.lock().await.clear();
            }
            _ = &mut shutdown => {
                log::debug!("Pty: received shutdown");
                break;
            }
        }
    }

    Ok(())
}

async fn reader_routine(
    reader: PtyReader,
    buffer: Arc<TokioMutex<VecDeque<u8>>>,
    channel: broadcast::Sender<Vec<u8>>,
) -> Result<(), Error> {
    let reader = Arc::new(StdMutex::new(reader));

    loop {
        let reader = reader.clone();

        let value = tokio::task::spawn_blocking::<_, Result<Vec<u8>, Error>>(move || {
            let mut buffer = [0; 8192];

            let size: usize = reader
                .lock()
                .handle_err(location!())?
                .read(&mut buffer)
                .handle_err(location!())?;

            Ok(Vec::from(&buffer[..size]))
        })
        .await
        .handle_err(location!())??;

        if value.is_empty() {
            break;
        }

        {
            let mut lock = buffer.lock().await;

            while lock.len() + value.len() > MAX_BUFFER_SIZE {
                lock.pop_front();
            }

            lock.extend(&value);
        }

        let _ = channel.send(value);
    }

    Ok(())
}

async fn writer_routine(
    writer: PtyWriter,
    channel: Arc<TokioMutex<mpsc::Receiver<Vec<u8>>>>,
) -> Result<(), Error> {
    let writer = Arc::new(StdMutex::new(writer));
    loop {
        let message = channel
            .lock()
            .await
            .recv()
            .await
            .ok_or("Channel closed unexpectedly")
            .handle_err(location!())?;

        let writer = writer.clone();

        tokio::task::spawn_blocking(move || writer.lock().unwrap().write_all(&message))
            .await
            .handle_err(location!())?
            .handle_err(location!())?;
    }
}
