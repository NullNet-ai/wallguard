use crate::datastore::DatastoreWrapper;
use crate::grpc_server::server::WallGuardImpl;
use crate::proto::wallguard::wall_guard_server::WallGuard;
use crate::proto::wallguard::{DeviceStatus, HeartbeatRequest, HeartbeatResponse};
use crate::tunnel::RAType;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use nullnet_libtoken::Token;
use std::time::Duration;
use tokio::sync::mpsc;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response};

impl WallGuardImpl {
    pub(crate) async fn heartbeat_impl(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<<WallGuardImpl as WallGuard>::HeartbeatStream>, Error> {
        let datastore = self.context.datastore.clone();
        let tunnel = self.context.tunnel.clone();
        let remote_address = request
            .remote_addr()
            .map_or_else(|| "Unknown".to_string(), |addr| addr.ip().to_string());

        let authenticate_request = request.into_inner();
        let mut auth_handler = AuthHandler::new(
            authenticate_request.app_id.clone(),
            authenticate_request.app_secret.clone(),
            datastore.clone(),
        );
        let token = auth_handler.obtain_token_safe().await?;
        let (_, token_info) = Self::authenticate(&token)?;
        let device_id = token_info.account.device.id;
        let device_version = authenticate_request.device_version;
        let device_uuid = authenticate_request.device_uuid;

        let status = datastore.device_status(device_id.clone(), &token).await?;
        if status == DeviceStatus::DsDraft {
            datastore
                .device_setup(
                    &token,
                    device_id.clone(),
                    device_version,
                    device_uuid,
                    remote_address,
                )
                .await?;
        }

        let (tx, rx) = mpsc::channel(6);

        tokio::spawn(async move {
            loop {
                if let Ok(token) = auth_handler.obtain_token_safe().await {
                    if let Ok(response) = datastore.heartbeat(&token, device_id.clone()).await {
                        let (remote_shell_enabled, remote_ui_enabled) = {
                            let tunnel = tunnel.lock().await;

                            let remote_shell_enabled = tunnel
                                .get_profile_by_device_id(&device_id, &RAType::Shell)
                                .await
                                .is_some();

                            let remote_ui_enabled = tunnel
                                .get_profile_by_device_id(&device_id, &RAType::UI)
                                .await
                                .is_some();

                            (remote_shell_enabled, remote_ui_enabled)
                        };

                        let response = HeartbeatResponse {
                            token,
                            status: response.status.into(),
                            remote_shell_enabled,
                            remote_ui_enabled,
                            is_monitoring_enabled: response.is_monitoring_enabled,
                        };
                        tx.send(Ok(response)).await.unwrap();
                    }
                };
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[derive(Debug)]
struct AuthHandler {
    app_id: String,
    app_secret: String,
    datastore: DatastoreWrapper,
    token: Option<Token>,
}

impl AuthHandler {
    pub fn new(app_id: String, app_secret: String, datastore: DatastoreWrapper) -> Self {
        Self {
            app_id,
            app_secret,
            datastore,
            token: None,
        }
    }

    pub async fn obtain_token_safe(&mut self) -> Result<String, Error> {
        if self.token.as_ref().is_none_or(Token::is_expired) {
            let jwt: String = self
                .datastore
                .login(self.app_id.clone(), self.app_secret.clone())
                .await?;

            let new_token = Token::from_jwt(jwt.as_str()).handle_err(location!())?;

            self.token = Some(new_token);
        }

        Ok(self.token.as_ref().unwrap().jwt.clone())
    }
}
