use tonic::Request;

use crate::datastore::DatastoreWrapper;
use nullnet_libdatastore::{
    Error as DSError, ErrorKind as DSErrorKind, GetByIdRequest, Params, Query,
};

impl DatastoreWrapper {
    pub async fn device_status(&self, device_id: String, token: String) -> Result<String, DSError> {
        let mut request = Request::new(GetByIdRequest {
            params: Some(Params {
                id: device_id,
                table: String::from("devices"),
            }),
            query: Some(Query {
                pluck: String::from("status"),
                durability: String::from("hard"),
            }),
        });

        Self::set_token_for_request(&mut request, &token)?;
        let response = self.inner.get_by_id(request).await?;

        let status = Self::internal_ds_parse_response_data(response.data)?;

        Ok(status.to_lowercase())
    }

    fn internal_ds_parse_response_data(data: String) -> Result<String, DSError> {
        serde_json::from_str::<serde_json::Value>(&data).map_err(|e| DSError {
            kind: DSErrorKind::ErrorRequestFailed,
            message: format!("Could not parse DS response: {}", e),
        })?.as_array()
        .and_then(|arr| arr.first())
        .and_then(|obj| obj.as_object())
        .and_then(|map| map.get("status"))
        .and_then(|v| v.as_str())
        .map(|status| status.to_string())
        .ok_or(DSError {
            kind: DSErrorKind::ErrorRequestFailed,
            message: String::from(
                "Failed to parse DS response. Either the format is unexpected or the device ID is missing",
            ),
        })
    }
}
