use crate::datastore::{
    Datastore, DeviceConfiguration,
    db_tables::DBTable,
    generated::{
        FilterCriteria, FilterOperator, GetByFilterParams, GetByFilterRequest,
        get_by_filter_request,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn obtain_config(
        &self,
        token: &str,
        device_id: &str,
    ) -> Result<Option<DeviceConfiguration>, Error> {
        let request = GetByFilterRequest {
            params: Some(GetByFilterParams {
                table: DBTable::DeviceConfigurations.into(),
                r#type: String::new(),
            }),
            body: Some(get_by_filter_request::GetByFilterBody {
                pluck: vec![
                    "id".to_string(),
                    "digest".to_string(),
                    "hostname".to_string(),
                    "device_id".to_string(),
                    "config_version".to_string(),
                ],
                advance_filters: vec![FilterCriteria {
                    r#type: "criteria".to_string(),
                    field: Some("device_id".to_string()),
                    entity: Some(DBTable::DeviceConfigurations.into()),
                    operator: Some(FilterOperator::Equal as i32),
                    values: vec![format!("\"{}\"", device_id)],
                    ..Default::default()
                }],
                limit: Some(1),
                order_by: Some("timestamp".to_string()),
                order_direction: Some("desc".to_string()),
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
            .ok_or("Empty config data")
            .handle_err(location!())?;
        let config =
            serde_json::from_value::<DeviceConfiguration>(first).handle_err(location!())?;

        Ok(Some(config))
    }
}
