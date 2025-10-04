use nullnet_liberror::{location, Error, ErrorHandler, Location};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, mpsc, Mutex};

use crate::remote_desktop::{
    messages::MessageHandler, screen_capturer::ScreenCapturer, screenshot::Screenshot,
};

mod messages;
mod screen_capturer;
mod screenshot;

// TODO: Change Vec<u8> to actual message type
type ConnectedClient = mpsc::Sender<Vec<u8>>;

type ClientsInner = Arc<Mutex<HashMap<u128, ConnectedClient>>>;

#[derive(Clone, Debug)]
pub struct RemoteDesktopManager {
    clients: ClientsInner,
    counter: u128,
    last_screenshot: Arc<Mutex<Screenshot>>,
    terminate: broadcast::Sender<()>,
    msg_handler: MessageHandler,
}

impl RemoteDesktopManager {
    pub fn new() -> Result<Self, Error> {
        let (terminate, _) = broadcast::channel(1);
        let msg_handler = MessageHandler::new()?;

        Ok(Self {
            terminate,
            clients: Default::default(),
            counter: 0,
            last_screenshot: Default::default(),
            msg_handler,
        })
    }

    pub async fn on_client_connected(&mut self, client: ConnectedClient) -> u128 {
        self.counter = self.counter.wrapping_add(1);

        let mut lock = self.clients.lock().await;

        if lock.is_empty() {
            let manager_clone = self.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                rt.block_on(async move {
                    capture_loop(manager_clone).await;
                });
            });
        }

        if let Ok(data) = self.last_screenshot.lock().await.as_webp() {
            let _ = client.send(data).await;
        }

        lock.insert(self.counter, client);

        self.counter
    }

    pub async fn on_client_disconnected(&mut self, id: u128) -> Result<(), Error> {
        self.clients
            .lock()
            .await
            .remove(&id)
            .ok_or(format!("No client by id {id}"))
            .handle_err(location!())?;

        if self.clients.lock().await.is_empty() {
            let _ = self.terminate.send(());
        }

        Ok(())
    }

    pub async fn on_client_message(&self, id: u128, message: Vec<u8>) -> Result<(), Error> {
        if !self.clients.lock().await.contains_key(&id) {
            return Err(format!("No client with ID {id}")).handle_err(location!());
        }

        self.msg_handler.on_message(message).await
    }
}

async fn capture_loop(manager: RemoteDesktopManager) {
    let mut terminate_receiver = manager.terminate.subscribe();
    tokio::select! {
        _ = terminate_receiver.recv() => {
            log::info!("RemoteDesktopManager: capture_loop received termination signal.")
        }
        retval = capture_loop_impl(manager) => {
            if let Err(err) = retval {
                log::error!("RemoteDesktopManager: capture_loop_impl resulted in error: {}", err.to_str());
            }
        }
    }
}

async fn capture_loop_impl(manager: RemoteDesktopManager) -> Result<(), Error> {
    let mut capturer = ScreenCapturer::new()?;

    loop {
        let screenshot = capturer.screenshot()?;

        let mut lock = manager.last_screenshot.lock().await;

        if !lock.compare(&screenshot) {
            let data = screenshot.as_webp()?;

            for client in manager.clients.lock().await.values() {
                let _ = client.send(data.clone());
            }

            *lock = screenshot;
        }

        // @TODO
        todo!()
        // tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
