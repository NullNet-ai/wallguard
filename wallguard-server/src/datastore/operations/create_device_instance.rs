use crate::datastore::{
    Datastore, DeviceInstance,
    db_tables::DBTable,
    generated::{CreateDeviceInstancesRequest, CreateParams, CreateQuery, DeviceInstances},
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn create_device_instance(
        &self,
        token: &str,
        instance: &DeviceInstance,
    ) -> Result<String, Error> {
        let request = CreateDeviceInstancesRequest {
            device_instances: Some(DeviceInstances {
                device_id: Some(instance.device_id.clone()),
                ..Default::default()
            }),
            params: Some(CreateParams {
                table: DBTable::DeviceInstances.into(),
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
            .create_device_instances(grpc_request)
            .await
            .handle_err(location!())?
            .into_inner();

        let id = response
            .data
            .and_then(|d| d.id)
            .ok_or("Missing 'id' in device instance response")
            .handle_err(location!())?;

        Ok(id)
    }
}
