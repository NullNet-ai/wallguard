use crate::datastore::DatastoreWrapper;
use nullnet_libdatastore::{Params, Query, ResponseData, UpdateRequest};
use nullnet_liberror::Error;
use serde_json::json;

impl DatastoreWrapper {
    pub async fn device_setup(
        &self,
        token: &str,
        device_id: String,
        device_version: String,
        device_uuid: String,
        device_address: String,
    ) -> Result<ResponseData, Error> {
        let request = UpdateRequest {
            params: Some(Params {
                table: String::from("devices"),
                id: device_id,
            }),
            query: Some(Query {
                pluck: String::from("id,code"),
                durability: String::from("soft"),
            }),
            body: json!({
                "device_version": device_version,
                "system_id": device_uuid,
                "ip_address": device_address,
                "is_connection_established": true,
                "status": "Active"
            })
            .to_string(),
        };

        let response = self.inner.clone().update(request, token).await?;

        Ok(response)
    }
}
