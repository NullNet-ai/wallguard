use nullnet_libdatastore::UpdateRequestBuilder;
use nullnet_liberror::Error;
use serde_json::json;

use crate::datastore::{Datastore, TunnelModel};

impl Datastore {
    pub async fn update_tunnel_accessed(
        &self,
        token: &str,
        tunnel_id: &str,
        performed_by_root: bool,
        timestamp: u64,
    ) -> Result<(), Error> {
        let (date, time) = crate::utilities::time::timestamp_to_datetime(timestamp.cast_signed());

        let update_type = if performed_by_root { "root" } else { "" };

        let request = UpdateDeviceTunnelsRequest {
            device_tunnel: Some(DeviceTunnels {
                last_access_time: Some(time.to_string()),
                last_access_date: Some(date.to_string()),
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
