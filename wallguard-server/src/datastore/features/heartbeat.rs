use chrono::Utc;
use serde_json::json;
use tonic::Request;

use crate::datastore::DatastoreWrapper;
use nullnet_libdatastore::{
    CreateParams, CreateRequest, Error as DSError, Query, Response as DSResponse,
};

impl DatastoreWrapper {
    pub async fn heartbeat(&self, token: &str, device_id: String) -> Result<DSResponse, DSError> {
        let mut request = Request::new(CreateRequest {
            params: Some(CreateParams {
                table: String::from("device_heartbeats"),
            }),
            query: Some(Query {
                pluck: String::new(),
            }),
            body: json!({
                "device_id": device_id,
                "timestamp": Utc::now().to_rfc3339(),
                "entity_prefix": String::from("HB")
            })
            .to_string(),
        });

        Self::set_token_for_request(&mut request, token)?;

        let response = self.inner.create(request).await?;

        Ok(response)
    }
}
