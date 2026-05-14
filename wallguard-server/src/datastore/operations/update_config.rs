use crate::datastore::{
    Datastore, DeviceConfiguration,
    db_tables::DBTable,
    generated::{
        DeviceConfigurations, UpdateDeviceConfigurationsRequest, UpdateParams, UpdateQuery,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn update_config(
        &self,
        token: &str,
        config_id: &str,
        config: &DeviceConfiguration,
    ) -> Result<(), Error> {
        let request = UpdateDeviceConfigurationsRequest {
            device_configuration: Some(DeviceConfigurations {
                digest: Some(config.digest.clone()),
                hostname: Some(config.hostname.clone()),
                device_id: Some(config.device_id.clone()),
                config_version: Some(config.version),
                ..Default::default()
            }),
            params: Some(UpdateParams {
                id: config_id.to_string(),
                table: DBTable::DeviceConfigurations.into(),
                r#type: String::new(),
            }),
            query: Some(UpdateQuery {
                pluck: String::new(),
            }),
        };

        let mut grpc_request = tonic::Request::new(request);
        grpc_request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", token)
                .parse()
                .handle_err(location!())?,
        );

        let _ = self
            .inner
            .clone()
            .update_device_configurations(grpc_request)
            .await
            .handle_err(location!())?;

        Ok(())
    }
}
