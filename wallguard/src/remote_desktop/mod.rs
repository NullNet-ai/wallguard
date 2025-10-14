use crate::remote_desktop::{
    messages::MessageHandler, screen_capturer::ScreenCapturer, screenshot::Screenshot,
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use openh264::OpenH264API;
use openh264::encoder::{Encoder, EncoderConfig};
use openh264::formats::YUVBuffer;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{Mutex, broadcast, mpsc};

mod client;
mod messages;
mod screen_capturer;
mod screenshot;

type ClientsInner = Arc<Mutex<HashMap<u128, client::Client>>>;

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

    pub async fn on_client_connected(&mut self, channel: mpsc::Sender<Vec<u8>>) -> u128 {
        let client_id = self.counter;
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

        lock.insert(client_id, client::Client::new(channel));

        client_id
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
    const TARGET_FPS: u64 = 24;
    const KEYFRAME_INTERVAL: u64 = 60;

    let mut capturer = ScreenCapturer::new()?;
    let target_frame_duration = Duration::from_millis(1000 / TARGET_FPS);

    let api = OpenH264API::from_source();

    let config = EncoderConfig::new();
    // config.set_bitrate_bps(2_000_000); // 2 Mbps
    // config.set_max_frame_rate(TARGET_FPS as f32);
    // config.enable_skip_frame(false);

    let mut encoder = Encoder::with_api_config(api, config).handle_err(location!())?;

    {
        let mut lock = manager.last_screenshot.lock().await;
        *lock = capturer.screenshot()?;
    }

    let mut frame_count: u64 = 0;

    loop {
        let frame_start = Instant::now();

        let screenshot = capturer.screenshot()?;
        if screenshot.is_empty() {
            tokio::time::sleep(target_frame_duration).await;
            continue;
        }

        let mut lock = manager.last_screenshot.lock().await;

        let yuv_frame = YUVBuffer::from_rgb8_source(screenshot.clone());

        if frame_count.is_multiple_of(KEYFRAME_INTERVAL) {
            encoder.force_intra_frame();
        }

        let encoded_frame = match encoder.encode(&yuv_frame) {
            Ok(bits) => bits.to_vec(),
            Err(e) => {
                log::warn!("Failed to encode frame: {:?}", e);
                *lock = screenshot;
                drop(lock);
                continue;
            }
        };

        if !encoded_frame.is_empty() {
            for client in manager.clients.lock().await.values() {
                let _ = client
                    .send(encoded_frame.clone(), target_frame_duration)
                    .await;
            }
        }

        *lock = screenshot;
        drop(lock);
        frame_count += 1;

        let elapsed = frame_start.elapsed();
        if elapsed < target_frame_duration {
            tokio::time::sleep(target_frame_duration - elapsed).await;
        }
    }
}
