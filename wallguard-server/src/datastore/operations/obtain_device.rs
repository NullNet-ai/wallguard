use crate::datastore::{
    Datastore, Device,
    db_tables::DBTable,
    generated::{
        FilterCriteria, FilterOperator, GetByFilterParams, GetByFilterRequest,
        get_by_filter_request,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn obtain_device_by_id(
        &self,
        token: &str,
        device_id: &str,
        performed_by_root: bool,
    ) -> Result<Option<Device>, Error> {
        let request = GetByFilterRequest {
            params: Some(GetByFilterParams {
                table: DBTable::Devices.into(),
                r#type: if performed_by_root {
                    "root".to_string()
                } else {
                    String::new()
                },
            }),
            body: Some(get_by_filter_request::GetByFilterBody {
                pluck: vec![
                    "id".to_string(),
                    "device_uuid".to_string(),
                    "is_traffic_monitoring_enabled".to_string(),
                    "is_config_monitoring_enabled".to_string(),
                    "is_telemetry_monitoring_enabled".to_string(),
                    "is_device_authorized".to_string(),
                    "device_category".to_string(),
                    "device_type".to_string(),
                    "device_name".to_string(),
                    "device_operating_system".to_string(),
                    "is_device_online".to_string(),
                    "organization_id".to_string(),
                    "device_version".to_string(),
                ],
                advance_filters: vec![FilterCriteria {
                    r#type: "criteria".to_string(),
                    field: Some("id".to_string()),
                    entity: Some(DBTable::Devices.into()),
                    operator: Some(FilterOperator::Equal as i32),
                    values: vec![format!("\"{}\"", device_id)],
                    ..Default::default()
                }],
                limit: Some(1),
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
            .get_by_filter(grpc_request)
            .await
            .handle_err(location!())?
            .into_inner();

        if response.count == 0 {
            return Ok(None);
        }

        let data: Vec<serde_json::Value> =
            serde_json::from_str(&response.data).handle_err(location!())?;
        let first = data
            .into_iter()
            .next()
            .ok_or("Empty device data")
            .handle_err(location!())?;
        let device = serde_json::from_value::<Device>(first).handle_err(location!())?;

        Ok(Some(device))
    }
}
