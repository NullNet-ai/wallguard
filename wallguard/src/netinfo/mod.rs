use std::time::Duration;

use nullnet_liberror::{ErrorHandler, Location, location};

use crate::{token_provider::TokenProvider, wg_server::WGServer};
use wallguard_common::protobuf::wallguard_service::ServicesMessage;

mod service;
mod sock;

const TIME_INTERVAL: Duration = Duration::from_secs(60);
// Services are a live snapshot, not a history log: on failure we don't persist
// and replay the stale list (it may include services that are down by the
// time we retry) — instead we back off briefly and re-scan for a fresh one.
const RETRY_INTERVAL: Duration = Duration::from_secs(5);

pub async fn monitor_services(interface: WGServer, token_provider: TokenProvider) {
    log::info!("Staring services monitoring ...");

    loop {
        let sockets = sock::get_sockets_info().await;
        let services = service::gather_info(sockets).await;

        if !services.is_empty() {
            let token = match token_provider
                .get()
                .await
                .ok_or("Failed to acquire token")
                .handle_err(location!())
            {
                Ok(t) => t,
                Err(e) => {
                    log::error!("monitor_services: token acquisition failed: {e:?}");
                    tokio::time::sleep(RETRY_INTERVAL).await;
                    continue;
                }
            };

            let message = ServicesMessage {
                services: services.into_iter().map(|value| value.into()).collect(),
                token,
            };

            if let Err(e) = interface.report_services(message).await {
                log::error!(
                    "monitor_services: reporting failed: {e:?}. Retrying shortly with a fresh scan."
                );
                tokio::time::sleep(RETRY_INTERVAL).await;
                continue;
            }
        }

        tokio::time::sleep(TIME_INTERVAL).await;
    }
}
