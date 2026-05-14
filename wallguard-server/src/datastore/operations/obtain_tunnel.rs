use crate::datastore::{
    Datastore, TunnelModel,
    db_tables::DBTable,
    generated::{
        FilterCriteria, FilterOperator, GetByFilterParams, GetByFilterRequest,
        get_by_filter_request,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn obtain_tunnel(
        &self,
        token: &str,
        tunnel_id: &str,
        performed_by_root: bool,
    ) -> Result<Option<TunnelModel>, Error> {
        let request = GetByFilterRequest {
            params: Some(GetByFilterParams {
                table: DBTable::DeviceTunnels.into(),
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
                    "tunnel_type".to_string(),
                    "service_id".to_string(),
                    "tunnel_status".to_string(),
                    "last_access_time".to_string(),
                    "last_access_date".to_string(),
                ],
                advance_filters: vec![FilterCriteria {
                    r#type: "criteria".to_string(),
                    field: Some("id".to_string()),
                    entity: Some(DBTable::DeviceTunnels.into()),
                    operator: Some(FilterOperator::Equal as i32),
                    values: vec![format!("\"{}\"", tunnel_id)],
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
            .ok_or("Empty tunnel data")
            .handle_err(location!())?;
        let tunnel = serde_json::from_value::<TunnelModel>(first).handle_err(location!())?;

        Ok(Some(tunnel))
    }
}
