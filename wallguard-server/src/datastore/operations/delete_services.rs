use crate::datastore::{
    Datastore, ServiceInfo,
    db_tables::DBTable,
    generated::{
        BatchDeleteDeviceServicesRequest, BatchDeleteParams, FilterCriteria, FilterOperator,
        batch_delete_device_services_request,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn delete_services(
        &self,
        token: &str,
        services: &[ServiceInfo],
    ) -> Result<(), Error> {
        if services.is_empty() {
            return Ok(());
        }

        let id_values: Vec<String> = services
            .iter()
            .map(|svc| format!("\"{}\"", svc.id))
            .collect();

        let request = BatchDeleteDeviceServicesRequest {
            params: Some(BatchDeleteParams {
                table: DBTable::DeviceServices.into(),
                r#type: String::new(),
            }),
            body: Some(batch_delete_device_services_request::BatchDeleteBody {
                advance_filters: vec![FilterCriteria {
                    r#type: "criteria".to_string(),
                    field: Some("id".to_string()),
                    entity: Some(DBTable::DeviceServices.into()),
                    operator: Some(FilterOperator::Contains as i32),
                    values: id_values,
                    ..Default::default()
                }],
            }),
        };

        let _ = self
            .inner
            .clone()
            .batch_delete_device_services(request)
            .await
            .handle_err(location!())?;

        Ok(())
    }
}
