use crate::datastore::{
    Datastore, DeviceConfiguration,
    db_tables::DBTable,
    generated::{
        CreateDeviceConfigurationsRequest, CreateParams, CreateQuery, DeviceConfigurations,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn create_config(
        &self,
        token: &str,
        config: &DeviceConfiguration,
    ) -> Result<String, Error> {
        let request = CreateDeviceConfigurationsRequest {
            device_configurations: Some(DeviceConfigurations {
                digest: Some(config.digest.clone()),
                hostname: Some(config.hostname.clone()),
                device_id: Some(config.device_id.clone()),
                config_version: Some(config.version),
                ..Default::default()
            }),
            params: Some(CreateParams {
                table: DBTable::DeviceConfigurations.into(),
                r#type: String::new(),
            }),
            query: Some(CreateQuery {
                pluck: "id".to_string(),
                ..Default::default()
            }),
        };

        let response = self
            .inner
            .clone()
            .create_device_configurations(request)
            .await
            .handle_err(location!())?
            .into_inner();

        if response.count != 1 {
            return Err("Failed to create device configuration").handle_err(location!());
        }

        let id = response
            .data
            .and_then(|d| d.id)
            .ok_or("Missing 'id' in device configuration response")
            .handle_err(location!())?;

        Ok(id)
    }
}
