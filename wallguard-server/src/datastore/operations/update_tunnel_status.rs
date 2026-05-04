use nullnet_libdatastore::UpdateRequestBuilder;
use nullnet_liberror::{Error, ErrorHandler, location, Location};
use crate::datastore::{Datastore, TunnelModel, TunnelStatus, db_tables::DBTable, generated::{DeviceTunnels, UpdateDeviceTunnelsRequest, UpdateParams, UpdateQuery}};

impl Datastore {
    pub async fn update_tunnel_status(
        &self,
        token: &str,
        tunnel_id: &str,
        status: TunnelStatus,
        performed_by_root: bool,
    ) -> Result<(), Error> {
        let update_type = if performed_by_root { "root" } else { "" };

        let request = UpdateDeviceTunnelsRequest {
            device_tunnel: Some(DeviceTunnels {
                tunnel_status: Some(status.to_string()),
                ..Default::default()
            }),
            params: Some(UpdateParams {
                id: tunnel_id.to_string(),
                table: DBTable::DeviceTunnels.into(),
                r#type: update_type.to_string(),
            }),
            query: Some(UpdateQuery {
                pluck: "".to_string()
            }),
        };

        let _ = self.inner.update_device_tunnels(request).await.handle_err(location!())?;

        Ok(())
    }
}
