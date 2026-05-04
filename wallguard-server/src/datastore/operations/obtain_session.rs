use crate::datastore::{
    Datastore, RemoteAccessSession,
    db_tables::DBTable,
    generated::{
        AggregationFilterParams, AggregationFilterRequest, AggregationOrder, FilterCriteria,
        FilterOperator, aggregation_filter_request,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn obtain_session(
        &self,
        token: &str,
        session_token: &str,
    ) -> Result<Option<RemoteAccessSession>, Error> {
        let request = AggregationFilterRequest {
            params: Some(AggregationFilterParams {
                r#type: String::new(),
            }),
            body: Some(aggregation_filter_request::AggregationFilterBody {
                entity: DBTable::RemoteAccessSessions.into(),
                advance_filters: vec![FilterCriteria {
                    r#type: "criteria".to_string(),
                    field: Some("remote_access_session".to_string()),
                    entity: Some(DBTable::RemoteAccessSessions.into()),
                    operator: Some(FilterOperator::Equal as i32),
                    values: vec![format!("\"{}\"", session_token)],
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

        let response = self
            .inner
            .clone()
            .aggregation_filter(request)
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
            .ok_or("Empty session data")
            .handle_err(location!())?;
        let session =
            serde_json::from_value::<RemoteAccessSession>(first).handle_err(location!())?;

        Ok(Some(session))
    }
}
