use crate::datastore::{
    Datastore, TunnelStatus,
    db_tables::DBTable,
    generated::{DeviceTunnels, UpdateDeviceTunnelsRequest, UpdateParams, UpdateQuery},
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn update_tunnel_status(
        &self,
        token: &str,
        tunnel_id: &str,
        status: TunnelStatus,
        performed_by_root: bool,
    ) -> Result<(), Error> {
        let request = UpdateDeviceTunnelsRequest {
            device_tunnel: Some(DeviceTunnels {
                tunnel_status: Some(status.to_string()),
                ..Default::default()
            }),
            params: Some(UpdateParams {
                id: tunnel_id.to_string(),
                table: DBTable::DeviceTunnels.into(),
                r#type: if performed_by_root {
                    "root".to_string()
                } else {
                    String::new()
                },
            }),
            query: Some(UpdateQuery {
                pluck: String::new(),
            }),
        };

        let mut grpc_request = tonic::Request::new(request);
        grpc_request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", token)
                .parse()
                .handle_err(location!())?,
        );

        let _ = self
            .inner
            .clone()
            .update_device_tunnels(grpc_request)
            .await
            .handle_err(location!())?;

        Ok(())
    }
}
