use crate::datastore::{
    Datastore,
    db_tables::DBTable,
    generated::{
        AggregationFilterParams, AggregationFilterRequest, FilterCriteria, FilterOperator,
        aggregation_filter_request,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn is_ip_info_stored(&self, ip: &str, token: &str) -> Result<bool, Error> {
        let request = AggregationFilterRequest {
            params: Some(AggregationFilterParams {
                r#type: String::new(),
            }),
            body: Some(aggregation_filter_request::AggregationFilterBody {
                entity: DBTable::IpInfos.into(),
                advance_filters: vec![FilterCriteria {
                    r#type: "criteria".to_string(),
                    field: Some("ip".to_string()),
                    entity: Some(DBTable::IpInfos.into()),
                    operator: Some(FilterOperator::Equal as i32),
                    values: vec![format!("\"{}\"", ip)],
                    ..Default::default()
                }],
                limit: Some(1),
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

        Ok(response.count > 0)
    }
}
