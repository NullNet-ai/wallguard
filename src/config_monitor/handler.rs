use super::request_impl::request_impl;
use crate::{authentication::AuthHandler, logger::Logger};
use nullnet_libconfmon::{Error, ErrorKind, Snapshot, WatcherHandler};

pub struct Handler {
    auth: AuthHandler,
    addr: String,
    port: u16,
}

impl Handler {
    pub fn new(addr: String, port: u16, auth: AuthHandler) -> Self {
        Self { auth, addr, port }
    }
}

impl WatcherHandler for Handler {
    async fn on_snapshot(
        &self,
        snapshot: Snapshot,
        state: nullnet_libconfmon::State,
    ) -> Result<(), nullnet_libconfmon::Error> {
        Logger::log(log::Level::Info, "Uploading configuration snapshot ...");

        let token = self
            .auth
            .obtain_token_safe()
            .await
            .map_err(|err_msg| Error {
                kind: ErrorKind::ErrorHandlingSnapshot,
                message: err_msg,
            })?;

        request_impl(&self.addr, self.port, snapshot, token, state)
            .await
            .map_err(|err_msg| Error {
                kind: ErrorKind::ErrorHandlingSnapshot,
                message: err_msg,
            })
    }

    async fn on_error(&self, error: Error) {
        Logger::log(
            log::Level::Error,
            format!("Error occured during configuration monitoring. {error}"),
        );
    }
}
