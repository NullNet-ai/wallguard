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
                status: Some(String::from("Active")),
                timestamp: Some(heartbeat.timestamp.clone()),
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
            .create_device_heartbeats(grpc_request)
            .await
            .handle_err(location!())?;

        Ok(())
    }
}
