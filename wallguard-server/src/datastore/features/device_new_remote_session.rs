use nullnet_libdatastore::{CreateBody, CreateParams, CreateRequest, Query};
use nullnet_liberror::Error;
use serde_json::json;

use crate::{datastore::DatastoreWrapper, tunnel::RAType};

impl DatastoreWrapper {
    pub async fn device_new_remote_session(
        &self,
        token: &str,
        device_id: String,
        remote_access_type: RAType,
    ) -> Result<(), Error> {
        let request = CreateRequest {
            params: Some(CreateParams {
                table: String::from("device_remote_access_sessions"),
            }),
            query: Some(Query {
                pluck: String::new(),
                durability: String::from("hard"),
            }),
            body: Some(CreateBody {
                record: json!({
                    "device_id": device_id,
                    "remote_access_type": remote_access_type.to_string(),
                })
                .to_string(),
                entity_prefix: String::from("RAS"),
            }),
        };

        let _ = self.inner.clone().create(request, token).await?;

        Ok(())
    }
}
