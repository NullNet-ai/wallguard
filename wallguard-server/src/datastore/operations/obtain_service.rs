use crate::datastore::{
    Datastore, ServiceInfo,
    db_tables::DBTable,
    generated::{
        FilterCriteria, FilterOperator, GetByFilterParams, GetByFilterRequest,
        get_by_filter_request,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn obtain_service(
        &self,
        token: &str,
        service_id: &str,
        performed_by_root: bool,
    ) -> Result<Option<ServiceInfo>, Error> {
        let request = GetByFilterRequest {
            params: Some(GetByFilterParams {
                table: DBTable::DeviceServices.into(),
                r#type: if performed_by_root {
                    "root".to_string()
                } else {
                    String::new()
                },
            }),
            body: Some(get_by_filter_request::GetByFilterBody {
                pluck: vec![
                    "id".to_string(),
                    "device_id".to_string(),
                    "address".to_string(),
                    "port".to_string(),
                    "protocol".to_string(),
                    "program".to_string(),
                ],
                advance_filters: vec![FilterCriteria {
                    r#type: "criteria".to_string(),
                    field: Some("id".to_string()),
                    entity: Some(DBTable::DeviceServices.into()),
                    operator: Some(FilterOperator::Equal as i32),
                    values: vec![format!("\"{}\"", service_id)],
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
            .ok_or("Empty service data")
            .handle_err(location!())?;
        let service = serde_json::from_value::<ServiceInfo>(first).handle_err(location!())?;

        Ok(Some(service))
    }
}
