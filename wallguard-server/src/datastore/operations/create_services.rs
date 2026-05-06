use crate::datastore::{
    Datastore, ServiceInfo,
    db_tables::DBTable,
    generated::{
        BatchInsertDeviceServicesRequest, BatchInsertParams, BatchInsertQuery, DeviceServices,
        batch_insert_device_services_request,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn create_services(
        &self,
        token: &str,
        services: &[ServiceInfo],
    ) -> Result<(), Error> {
        if services.is_empty() {
            return Ok(());
        }

        let records: Vec<DeviceServices> = services
            .iter()
            .map(|svc| DeviceServices {
                device_id: Some(svc.device_id.clone()),
                address: Some(svc.address.clone()),
                port: Some(svc.port as i32),
                protocol: Some(svc.protocol.clone()),
                program: Some(svc.program.clone()),
                ..Default::default()
            })
            .collect();

        let request = BatchInsertDeviceServicesRequest {
            params: Some(BatchInsertParams {
                table: DBTable::DeviceServices.into(),
                r#type: String::new(),
            }),
            query: Some(BatchInsertQuery {
                pluck: String::new(),
            }),
            body: Some(batch_insert_device_services_request::BatchBody {
                device_services: records,
            }),
        };

        let mut grpc_request = tonic::Request::new(request);
        grpc_request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", token).parse().handle_err(location!())?,
        );

        let _ = self
            .inner
            .clone()
            .batch_insert_device_services(grpc_request)
            .await
            .handle_err(location!())?;

        Ok(())
    }
}
