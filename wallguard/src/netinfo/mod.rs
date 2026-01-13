use std::time::Duration;

use nullnet_liberror::{Error, ErrorHandler, Location, location};

use crate::{token_provider::TokenProvider, wg_server::WGServer};
use wallguard_common::protobuf::wallguard_service::ServicesMessage;

mod service;
mod sock;

const TIME_INTERVAL: Duration = Duration::from_secs(60);

pub async fn monitor_services(
    interface: WGServer,
    token_provider: TokenProvider,
) -> Result<(), Error> {
    log::info!("Staring services monitoring ...");

    loop {
        let sockets = sock::get_sockets_info().await;
        let services = service::gather_info(&sockets).await;

        if !services.is_empty() {
            let message = ServicesMessage {
                services: services.into_iter().map(|value| value.into()).collect(),
                token: token_provider
                    .get()
                    .await
                    .ok_or("Failed to acquire token")
                    .handle_err(location!())?,
            };

            interface
                .get_interface(false)
                .await?
                .report_services(message)
                .await?;
        }

        tokio::time::sleep(TIME_INTERVAL).await;
    }
}
