use crate::cli::Args;
use handler::Handler;
use nullnet_libconfmon::{Error, Watcher};
use std::sync::Arc;
use tokio::sync::RwLock;

mod handler;
mod request_impl;

static DEFAULT_POLL_INTERVAL_MS: u64 = 500;

pub struct ConfigurationMonitor {
    watcher: Watcher<Handler>,
}

impl ConfigurationMonitor {
    pub async fn new(
        args: &Args,
        token: Arc<RwLock<String>>,
        poll_interval: Option<u64>,
    ) -> Result<Self, Error> {
        let handler = Handler::new(args.addr.clone(), args.port, token);
        let watcher = nullnet_libconfmon::make_watcher(
            &args.target,
            poll_interval.unwrap_or(DEFAULT_POLL_INTERVAL_MS),
            handler,
        )
        .await?;

        Ok(Self { watcher })
    }

    pub async fn upload_current(&self) -> Result<(), String> {
        self.watcher
            .force_capture_and_dispatch()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn watch(&mut self) {
        self.watcher.watch().await;
    }
}
