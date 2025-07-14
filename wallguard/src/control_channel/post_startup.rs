use crate::{context::Context, token_provider::RetrievalStrategy};
use wallguard_common::protobuf::wallguard_service::DeviceSettingsRequest;
use std::time::Duration;

pub async fn post_startup(context: Context) {
    let timeout = Duration::from_secs(10);

    let token = context
        .token_provider
        .obtain(RetrievalStrategy::Await(timeout))
        .await;

    if token.is_none() {
        log::error!("Failed to obtain auth token");
        return;
    }

    let Ok(response) = context
        .server
        .get_device_settings(DeviceSettingsRequest {
            token: token.unwrap(),
        })
        .await
    else {
        log::error!("Failed to fetch device settings");
        return;
    };

    if response.config_monitoring {
        context
            .transmission_manager
            .lock()
            .await
            .start_sysconf_monitroing();
    }

    if response.telemetry_monitoring {
        context
            .transmission_manager
            .lock()
            .await
            .start_resource_monitoring();
    }

    if response.traffic_monitoring {
        context
            .transmission_manager
            .lock()
            .await
            .start_packet_capture();
    }
}
