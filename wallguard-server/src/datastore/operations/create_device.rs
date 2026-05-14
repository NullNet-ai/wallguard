use crate::datastore::{
    Datastore, Device,
    db_tables::DBTable,
    generated::{CreateDevicesRequest, CreateParams, CreateQuery, Devices},
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn create_device(&self, token: &str, device: &Device) -> Result<String, Error> {
        let request = CreateDevicesRequest {
            devices: Some(Devices {
                device_uuid: Some(device.uuid.clone()),
                is_traffic_monitoring_enabled: Some(device.traffic_monitoring),
                is_config_monitoring_enabled: Some(device.sysconf_monitoring),
                is_telemetry_monitoring_enabled: Some(device.telemetry_monitoring),
                is_device_authorized: Some(device.authorized),
                device_category: Some(device.category.clone()),
                device_type: Some(device.r#type.clone()),
                device_name: Some(device.name.clone()),
                device_operating_system: Some(device.os.clone()),
                is_device_online: Some(device.online),
                organization_id: Some(device.organization.clone()),
                device_version: Some(device.version.clone()),
                ..Default::default()
            }),
            params: Some(CreateParams {
                table: DBTable::Devices.into(),
                r#type: String::new(),
            }),
            query: Some(CreateQuery {
                pluck: "id".to_string(),
                ..Default::default()
            }),
        };

        let mut grpc_request = tonic::Request::new(request);
        grpc_request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", token)
                .parse()
                .handle_err(location!())?,
        );

        let response = self
            .inner
            .clone()
            .create_devices(grpc_request)
            .await
            .handle_err(location!())?
            .into_inner();

        let id = response
            .data
            .and_then(|d| d.id)
            .ok_or("Missing 'id' in device response")
            .handle_err(location!())?;

        Ok(id)
    }
}
