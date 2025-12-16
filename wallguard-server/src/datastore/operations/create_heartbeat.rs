use nullnet_libdatastore::CreateRequestBuilder;
use nullnet_liberror::Error;
use serde_json::json;

use crate::datastore::{Datastore, HeartbeatModel};

impl Datastore {
    pub async fn create_heartbeat(
        &self,
        token: &str,
        heartbeat: &HeartbeatModel,
    ) -> Result<(), Error> {
        let json = json!(heartbeat);

        let request = CreateRequestBuilder::new()
            .pluck(HeartbeatModel::pluck())
            .table(HeartbeatModel::table())
            .record(json.to_string())
            .build();

        let _ = self.inner.clone().create(request, token).await?;

        Ok(())
    }
}
