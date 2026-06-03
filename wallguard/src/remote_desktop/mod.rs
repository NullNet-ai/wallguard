use crate::remote_desktop::{messages::MessageHandler, screen_capturer::ScreenCapturer};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use openh264::OpenH264API;
use openh264::encoder::{Encoder, EncoderConfig};
use openh264::formats::YUVBuffer;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::{Mutex, RwLock, broadcast, mpsc};

mod client;
mod messages;
mod screen_capturer;
mod screenshot;
#[cfg(target_os = "linux")]
mod uinput_handler;

type ClientsInner = Arc<Mutex<HashMap<u128, client::Client>>>;

#[derive(Clone, Debug)]
pub struct RemoteDesktopManager {
    clients: ClientsInner,
    counter: Arc<RwLock<u128>>,
    /// Set to true when a new viewer connects so the encode loop sends a keyframe
    /// immediately, giving them a complete picture before the periodic interval.
    force_keyframe: Arc<AtomicBool>,
    terminate: broadcast::Sender<()>,
    msg_handler: MessageHandler,
    capturer: Arc<Mutex<ScreenCapturer>>,
}

impl RemoteDesktopManager {
    pub fn new() -> Result<Self, Error> {
        let (terminate, _) = broadcast::channel(1);
        let msg_handler = MessageHandler::new()?;
        let capturer = ScreenCapturer::new()?;

        Ok(Self {
            terminate,
            clients: Default::default(),
            counter: Default::default(),
            force_keyframe: Arc::new(AtomicBool::new(false)),
            msg_handler,
            capturer: Arc::new(Mutex::new(capturer)),
        })
    }

    pub async fn on_client_connected(&mut self, channel: mpsc::Sender<Vec<u8>>) -> u128 {
        let client_id = {
            let mut counter = self.counter.write().await;
            let id = *counter;
            *counter = id.wrapping_add(1);
            id
        };

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

        // New viewer: force an intra frame so they get a complete picture immediately
        // rather than waiting up to KEYFRAME_INTERVAL frames for the next periodic one.
        self.force_keyframe.store(true, Ordering::Relaxed);

        lock.insert(client_id, client::Client::new(channel));

        log::debug!("Client with ID {client_id} has just connected");

        client_id
    }

    pub async fn on_client_disconnected(&mut self, id: u128) -> Result<(), Error> {
        let mut lock = self.clients.lock().await;

        lock.remove(&id)
            .ok_or(format!("No client by id {id}"))
            .handle_err(location!())?;

        if lock.is_empty() {
            let _ = self.terminate.send(());
        }

        log::debug!("Client with ID {id} has just disconnected");

        Ok(())
    }

    pub async fn on_client_message(&self, id: u128, message: Vec<u8>) -> Result<(), Error> {
        if message.is_empty() {
            return Ok(());
        }

        if !self.clients.lock().await.contains_key(&id) {
            return Err(format!("No client with ID {id}")).handle_err(location!());
        }

        self.msg_handler.on_message(message).await
    }
}

async fn capture_loop(manager: RemoteDesktopManager) {
    let cleanup = manager.clone();
    let mut terminate_receiver = manager.terminate.subscribe();
    tokio::select! {
        _ = terminate_receiver.recv() => {
            log::info!("RemoteDesktopManager: capture_loop received termination signal.")
        }
        retval = capture_loop_impl(manager) => {
            if let Err(err) = retval {
                log::error!("RemoteDesktopManager: capture_loop_impl resulted in error: {}", err.to_str());

                // Drop every client's mpsc sender so that `system_to_stream` in
                // `OpenRemoteDesktopSessionCommand` sees a closed channel and
                // returns.  That ends the `tokio::select!` there, drops the TCP
                // `TunnelInstance`, and the server-side `InternalRelay` detects
                // the EOF → drops the broadcast sender → `relay_rd_to_user` gets
                // `RecvError::Closed` → WebSocket closes cleanly instead of
                // hanging until the browser times it out.
                cleanup.clients.lock().await.clear();
            }
        }
    }
}

async fn capture_loop_impl(manager: RemoteDesktopManager) -> Result<(), Error> {
    const TARGET_FPS: u64 = 24;
    const KEYFRAME_INTERVAL: u64 = 60;

    let target_frame_duration = Duration::from_millis(1000 / TARGET_FPS);

    let api = OpenH264API::from_source();
    let config = EncoderConfig::new().skip_frames(false);
    let mut encoder = Encoder::with_api_config(api, config).handle_err(location!())?;
    let mut frame_count: u64 = 0;

    loop {
        let frame_start = Instant::now();

        // Skip capture entirely when nobody is watching.
        if manager.clients.lock().await.is_empty() {
            tokio::time::sleep(target_frame_duration).await;
            continue;
        }

        let screenshot = manager.capturer.lock().await.screenshot()?;
        if screenshot.is_empty() {
            tokio::time::sleep(target_frame_duration).await;
            continue;
        }

        // Force an intra frame on the periodic interval or when a new viewer just
        // joined (swap clears the flag atomically so we only do it once).
        if frame_count.is_multiple_of(KEYFRAME_INTERVAL)
            || manager.force_keyframe.swap(false, Ordering::Relaxed)
        {
            encoder.force_intra_frame();
        }

        // screenshot is moved here; no clone needed since we no longer cache it.
        let yuv_frame = YUVBuffer::from_rgb8_source(screenshot);

        let encoded = match encoder.encode(&yuv_frame) {
            Ok(bits) => bits.to_vec(),
            Err(e) => {
                log::warn!("Failed to encode frame: {:?}", e);
                frame_count += 1;
                continue;
            }
        };

        if !encoded.is_empty() {
            let clients = manager.clients.lock().await;
            for client in clients.values() {
                let _ = client.send(encoded.clone(), target_frame_duration).await;
            }
        }

        frame_count += 1;

        let elapsed = frame_start.elapsed();
        if elapsed < target_frame_duration {
            tokio::time::sleep(target_frame_duration - elapsed).await;
        }
    }
}
