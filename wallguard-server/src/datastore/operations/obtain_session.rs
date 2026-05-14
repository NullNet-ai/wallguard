use crate::datastore::{
    Datastore, RemoteAccessSession,
    db_tables::DBTable,
    generated::{
        FilterCriteria, FilterOperator, GetByFilterParams, GetByFilterRequest,
        get_by_filter_request,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn obtain_session(
        &self,
        token: &str,
        session_token: &str,
    ) -> Result<Option<RemoteAccessSession>, Error> {
        let request = GetByFilterRequest {
            params: Some(GetByFilterParams {
                table: DBTable::RemoteAccessSessions.into(),
                r#type: String::new(),
            }),
            body: Some(get_by_filter_request::GetByFilterBody {
                pluck: vec![
                    "id".to_string(),
                    "device_id".to_string(),
                    "instance_id".to_string(),
                    "remote_access_session".to_string(),
                    "remote_access_type".to_string(),
                    "remote_access_local_addr".to_string(),
                    "remote_access_local_port".to_string(),
                    "remote_access_local_protocol".to_string(),
                ],
                advance_filters: vec![FilterCriteria {
                    r#type: "criteria".to_string(),
                    field: Some("remote_access_session".to_string()),
                    entity: Some(DBTable::RemoteAccessSessions.into()),
                    operator: Some(FilterOperator::Equal as i32),
                    values: vec![format!("\"{}\"", session_token)],
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
            .ok_or("Empty session data")
            .handle_err(location!())?;
        let session =
            serde_json::from_value::<RemoteAccessSession>(first).handle_err(location!())?;

        Ok(Some(session))
    }
}
