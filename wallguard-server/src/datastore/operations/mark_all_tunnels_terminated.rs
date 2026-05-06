use crate::datastore::{
    Datastore, TunnelStatus,
    db_tables::DBTable,
    generated::{
        BatchUpdateDeviceTunnelsRequest, BatchUpdateParams, DeviceTunnels, FilterCriteria,
        FilterOperator, batch_update_device_tunnels_request,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn mark_all_tunnels_terminated(
        &self,
        token: &str,
        performed_by_root: bool,
    ) -> Result<(), Error> {
        let request = BatchUpdateDeviceTunnelsRequest {
            params: Some(BatchUpdateParams {
                table: DBTable::DeviceTunnels.into(),
                r#type: if performed_by_root {
                    "root".to_string()
                } else {
                    String::new()
                },
            }),
            body: Some(batch_update_device_tunnels_request::BatchUpdateBody {
                advance_filters: vec![FilterCriteria {
                    r#type: "criteria".to_string(),
                    field: Some("status".to_string()),
                    entity: Some(DBTable::DeviceTunnels.into()),
                    operator: Some(FilterOperator::Equal as i32),
                    values: vec!["\"Active\"".to_string()],
                    ..Default::default()
                }],
                updates: Some(DeviceTunnels {
                    tunnel_status: Some(TunnelStatus::Terminated.to_string()),
                    ..Default::default()
                }),
            }),
        };

        let mut grpc_request = tonic::Request::new(request);
        grpc_request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", token).parse().handle_err(location!())?,
        );

        let _ = self
            .inner
            .clone()
            .batch_update_device_tunnels(grpc_request)
            .await
            .handle_err(location!())?;

        Ok(())
    }
}
