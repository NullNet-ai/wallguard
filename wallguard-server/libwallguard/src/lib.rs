use nullnet_liberror::{Error, ErrorHandler, Location, location};
pub use proto::wallguard_authorization::*;
pub use proto::wallguard_commands::wall_guard_command::*;
pub use proto::wallguard_commands::*;
use proto::wallguard_service::wall_guard_client::*;
pub use proto::wallguard_service::*;
use std::time::Duration;
pub use tonic::Streaming;
use tonic::transport::Channel;
use tonic::{Request, Status};

mod proto;

#[derive(Clone, Debug)]
pub struct WallGuardGrpcInterface {
    client: WallGuardClient<Channel>,
}

impl WallGuardGrpcInterface {
    #[allow(clippy::missing_panics_doc)]
    pub async fn new(addr: &str, port: u16) -> Result<Self, Error> {
        let addr = format!("http://{addr}:{port}");

        let channel = Channel::from_shared(addr)
            .expect("Failed to parse address")
            .timeout(Duration::from_secs(10))
            .keep_alive_timeout(Duration::from_secs(10))
            .connect()
            .await
            .handle_err(location!())?;

        let client = WallGuardClient::new(channel).max_decoding_message_size(50 * 1024 * 1024);

        Ok(Self { client })
    }

    pub async fn request_control_channel(
        &self,
        app_id: &str,
        app_secret: &str,
    ) -> Result<Streaming<WallGuardCommand>, Status> {
        let request = Request::new(ControlChannelRequest {
            app_id: app_id.into(),
            app_secret: app_secret.into(),
        });

        self.client
            .clone()
            .control_channel(request)
            .await
            .map(tonic::Response::into_inner)
    }

    pub async fn authorization_request(
        &self,
        uuid: &str,
        org_id: &str,
    ) -> Result<Streaming<AuthorizationStatus>, Status> {
        let request = Request::new(AuthorizationRequest {
            organization_id: org_id.into(),
            device_uuid: uuid.into(),
        });

        self.client
            .clone()
            .device_authorization(request)
            .await
            .map(tonic::Response::into_inner)
    }
}
