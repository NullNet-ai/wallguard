use crate::datastore::{
    Datastore, DeviceConfiguration,
    db_tables::DBTable,
    generated::{
        AggregationFilterParams, AggregationFilterRequest, AggregationOrder, FilterCriteria,
        FilterOperator, aggregation_filter_request,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn obtain_config(
        &self,
        token: &str,
        device_id: &str,
    ) -> Result<Option<DeviceConfiguration>, Error> {
        let request = AggregationFilterRequest {
            params: Some(AggregationFilterParams {
                r#type: String::new(),
            }),
            body: Some(aggregation_filter_request::AggregationFilterBody {
                entity: DBTable::DeviceConfigurations.into(),
                advance_filters: vec![FilterCriteria {
                    r#type: "criteria".to_string(),
                    field: Some("device_id".to_string()),
                    entity: Some(DBTable::DeviceConfigurations.into()),
                    operator: Some(FilterOperator::Equal as i32),
                    values: vec![format!("\"{}\"", device_id)],
                    ..Default::default()
                }],
                limit: Some(1),
                order: Some(AggregationOrder {
                    order_by: "timestamp".to_string(),
                    order_direction: "desc".to_string(),
                }),
                ..Default::default()
            }),
        };

        let mut grpc_request = tonic::Request::new(request);
        grpc_request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", token).parse().handle_err(location!())?,
        );

        let response = self
            .inner
            .clone()
            .aggregation_filter(grpc_request)
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
