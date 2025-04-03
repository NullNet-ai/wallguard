use crate::datastore::DatastoreWrapper;
use nullnet_libdatastore::{GetByIdRequest, Params, Query};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl DatastoreWrapper {
    pub async fn device_fetch_webgui_protocol(
        &self,
        device_id: &str,
        token: &str,
    ) -> Result<String, Error> {
        let request = GetByIdRequest {
            params: Some(Params {
                id: device_id.to_owned(),
                table: String::from("devices"),
            }),
            query: Some(Query {
                pluck: String::from("device_gui_protocol"),
                durability: String::from("hard"),
            }),
        };

        let response = self.inner.clone().get_by_id(request, token).await?;
        Self::internal_fp_parse_response_data(&response.data)
    }

    fn internal_fp_parse_response_data(data: &str) -> Result<String, Error> {
        serde_json::from_str::<serde_json::Value>(data)
            .handle_err(location!())?
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|obj| obj.as_object())
            .and_then(|map| map.get("device_gui_protocol"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or("Failed to parse response")
            .handle_err(location!())
    }
}
