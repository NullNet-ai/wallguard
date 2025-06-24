use crate::control_service::service::WallGuardService;
use crate::protocol::wallguard_service::DeviceSettingsRequest;
use crate::protocol::wallguard_service::DeviceSettingsResponse;
use nullnet_libtoken::Token;
use tonic::{Request, Response, Status};

impl WallGuardService {
    pub(crate) async fn get_device_settings_impl(
        &self,
        request: Request<DeviceSettingsRequest>,
    ) -> Result<Response<DeviceSettingsResponse>, Status> {
        let token = Token::from_jwt(&request.into_inner().token)
            .map_err(|_| Status::internal("Malformed JWT token"))?;

        let device = self
            .ensure_device_exists_and_authrorized(&token)
            .await
            .map_err(|err| Status::internal(err.to_str()))?;

        let response = DeviceSettingsResponse {
            config_monitoring: device.sysconf_monitoring,
            traffic_monitoring: device.traffic_monitoring,
            telemetry_monitoring: device.telemetry_monitoring,
        };

        Ok(Response::new(response))
    }
}
