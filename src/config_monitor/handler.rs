use super::request_impl::request_impl;
use nullnet_libconfmon::{Error, ErrorKind, FileData, InterfaceSnapshot, Snapshot, WatcherHandler};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Handler {
    token: Arc<RwLock<String>>,
    addr: String,
    port: u16,
}

impl Handler {
    pub fn new(addr: String, port: u16, token: Arc<RwLock<String>>) -> Self {
        Self { token, addr, port }
    }

    fn map_error<T>(msg: T) -> Error
    where
        T: ToString,
    {
        Error {
            kind: ErrorKind::ErrorHandlingSnapshot,
            message: msg.to_string(),
        }
    }
}

impl WatcherHandler for Handler {
    async fn on_snapshot(
        &self,
        mut snapshot: Snapshot,
        state: nullnet_libconfmon::State,
    ) -> Result<(), nullnet_libconfmon::Error> {
        log::info!("Uploading configuration snapshot ...");

        let ifaces_data = InterfaceSnapshot::take_all();
        let blob = InterfaceSnapshot::serialize_snapshot(&ifaces_data).map_err(Self::map_error)?;

        snapshot.push(FileData {
            filename: "#NetworkInterfaces".to_string(),
            content: blob,
        });

        request_impl(&self.addr, self.port, snapshot, self.token.clone(), state)
            .await
            .map_err(Self::map_error)
    }

    async fn on_error(&self, error: Error) {
        log::error!("Error occured during configuration monitoring. {error}");
    }
}
