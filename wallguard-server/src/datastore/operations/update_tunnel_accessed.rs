use crate::datastore::{
    Datastore,
    db_tables::DBTable,
    generated::{DeviceTunnels, UpdateDeviceTunnelsRequest, UpdateParams, UpdateQuery},
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn update_tunnel_accessed(
        &self,
        token: &str,
        tunnel_id: &str,
        performed_by_root: bool,
        timestamp: u64,
    ) -> Result<(), Error> {
        let (date, time) =
            crate::utilities::time::timestamp_to_datetime(timestamp.cast_signed());

        let request = UpdateDeviceTunnelsRequest {
            device_tunnel: Some(DeviceTunnels {
                last_access_time: Some(time.to_string()),
                last_access_date: Some(date.to_string()),
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

        let _ = self
            .inner
            .clone()
            .update_device_tunnels(request)
            .await
            .handle_err(location!())?;

        Ok(())
    }
}
