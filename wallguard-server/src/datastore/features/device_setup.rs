use serde_json::json;
use tonic::Request;

use crate::datastore::DatastoreWrapper;
use nullnet_libdatastore::{
    Error as DSError, Params, Query, Response as DSResponse, UpdateRequest,
};

impl DatastoreWrapper {
    pub async fn device_setup(
        &self,
        token: String,
        device_id: String,
        device_version: String,
        device_uuid: String,
        device_hostname: String,
        device_address: String,
    ) -> Result<DSResponse, DSError> {
        let mut request = Request::new(UpdateRequest {
            params: Some(Params {
                table: String::from("devices"),
                id: device_id,
            }),
            query: Some(Query {
                pluck: String::from("id,code"),
                durability: String::from("hard"),
            }),
            body: json!({
                "device_version": device_version,
                "hostname": device_hostname,
                "system_id": device_uuid,
                "ip_address": device_address,
                "is_connection_established": true,
                "status": "Active"
            })
            .to_string(),
        });

        Self::set_token_for_request(&mut request, &token)?;

        let response = self.inner.update(request).await?;

        Ok(response)
    }
}
