use crate::datastore::{
    Datastore,
    db_tables::DBTable,
    generated::{
        FilterCriteria, FilterOperator, GetByFilterParams, GetByFilterRequest,
        get_by_filter_request,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn is_ip_info_stored(&self, ip: &str, token: &str) -> Result<bool, Error> {
        let request = GetByFilterRequest {
            params: Some(GetByFilterParams {
                table: DBTable::IpInfos.into(),
                r#type: String::new(),
            }),
            body: Some(get_by_filter_request::GetByFilterBody {
                pluck: vec!["id".to_string()],
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

        Ok(response.count > 0)
    }
}
