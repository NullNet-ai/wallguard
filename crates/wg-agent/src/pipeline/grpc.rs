use std::time::Duration;

use tokio::sync::broadcast;
use tonic::transport::Channel;
use tracing::{info, warn};

use crate::config::Config;
use crate::proto::data::data_service_client::DataServiceClient;

const RECONNECT_DELAY: Duration = Duration::from_secs(5);

/// Connect to the Data gRPC service, retrying on failure until either a
/// connection succeeds or the shutdown signal fires.
///
/// Returns `None` when the shutdown signal fires before a connection is made.
/// Returns `Some(client)` on success, at which point the caller runs its
/// session and calls this again to reconnect if needed.
pub async fn connect_with_retry(
    config:   &Config,
    shutdown: &mut broadcast::Receiver<()>,
) -> Option<DataServiceClient<Channel>> {
    loop {
        if shutdown.try_recv().is_ok() {
            return None;
        }

        match crate::tls::build_grpc_channel(config, config.grpc_endpoint()).await {
            Ok(channel) => {
                info!("data gRPC connected");
                return Some(DataServiceClient::new(channel));
            }
            Err(e) => {
                warn!("data gRPC connect failed: {e:#}");
                tokio::select! {
                    _ = shutdown.recv()                      => return None,
                    _ = tokio::time::sleep(RECONNECT_DELAY) => {}
                }
            }
        }
    }
}
