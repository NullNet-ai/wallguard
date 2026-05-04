use crate::datastore::{
    Datastore, HeartbeatModel,
    db_tables::DBTable,
    generated::{CreateDeviceHeartbeatsRequest, CreateParams, CreateQuery, DeviceHeartbeats},
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn create_heartbeat(
        &self,
        token: &str,
        heartbeat: &HeartbeatModel,
    ) -> Result<(), Error> {
        let request = CreateDeviceHeartbeatsRequest {
            device_heartbeats: Some(DeviceHeartbeats {
                device_id: Some(heartbeat.device_id.clone()),
                ..Default::default()
            }),
            params: Some(CreateParams {
                table: DBTable::Heartbeats.into(),
                r#type: String::new(),
            }),
            query: Some(CreateQuery {
                pluck: String::new(),
                ..Default::default()
            }),
        };

        let _ = self
            .inner
            .clone()
            .create_device_heartbeats(request)
            .await
            .handle_err(location!())?;

        Ok(())
    }
}
