use crate::datastore::{
    Datastore, InstallationCode,
    db_tables::DBTable,
    generated::{
        AggregationFilterParams, AggregationFilterRequest, AggregationOrder, FilterCriteria,
        FilterOperator, aggregation_filter_request,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn obtain_installation_code(
        &self,
        code: &str,
        token: &str,
    ) -> Result<Option<InstallationCode>, Error> {
        let request = AggregationFilterRequest {
            params: Some(AggregationFilterParams {
                r#type: "root".to_string(),
            }),
            body: Some(aggregation_filter_request::AggregationFilterBody {
                entity: DBTable::InstallationCodes.into(),
                advance_filters: vec![FilterCriteria {
                    r#type: "criteria".to_string(),
                    field: Some("token".to_string()),
                    entity: Some(DBTable::InstallationCodes.into()),
                    operator: Some(FilterOperator::Equal as i32),
                    values: vec![format!("\"{}\"", code)],
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
            format!("Bearer {}", token)
                .parse()
                .handle_err(location!())?,
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
            .ok_or("Empty installation code data")
            .handle_err(location!())?;
        let installation_code =
            serde_json::from_value::<InstallationCode>(first).handle_err(location!())?;

        Ok(Some(installation_code))
    }
}
