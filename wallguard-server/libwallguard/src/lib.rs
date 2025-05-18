use std::time::Duration;
pub use tonic::Streaming;
use tonic::transport::Channel;
use tonic::{Request, Status};

pub use proto::wallguard_commands::wall_guard_command::*;
pub use proto::wallguard_commands::*;

use proto::wallguard_service::wall_guard_client::*;
pub use proto::wallguard_service::*;

mod proto;

#[derive(Clone, Debug)]
pub struct WallGuardGrpcInterface {
    client: WallGuardClient<Channel>,
}

impl WallGuardGrpcInterface {
    #[allow(clippy::missing_panics_doc)]
    pub async fn new(addr: &str, port: u16) -> Self {
        let s = format!("http://{addr}:{port}");

        let Ok(channel) = Channel::from_shared(s)
            .expect("Failed to parse address")
            .timeout(Duration::from_secs(10))
            .connect()
            .await
        else {
            // @TODO: Perhaps jusr return an error and let client deal with it ?
            log::warn!("Failed to connect to the server. Retrying in 10 seconds...");
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            return Box::pin(WallGuardGrpcInterface::new(addr, port)).await;
        };

        Self {
            client: WallGuardClient::new(channel).max_decoding_message_size(50 * 1024 * 1024),
        }
    }

    pub async fn request_control_channel(
        &mut self,
        app_id: &str,
        app_secret: &str,
    ) -> Result<Streaming<WallGuardCommand>, Status> {
        let request = Request::new(ControlChannelRequest {
            app_id: app_id.into(),
            app_secret: app_secret.into(),
        });

        self.client
            .control_channel(request)
            .await
            .map(tonic::Response::into_inner)
    }
}
