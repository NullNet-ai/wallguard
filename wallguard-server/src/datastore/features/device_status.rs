use super::utils::map_status_value_to_enum;
use crate::{datastore::DatastoreWrapper, proto::wallguard::DeviceStatus};
use nullnet_libdatastore::{GetByIdRequest, Params, Query};
use nullnet_liberror::{location, Error, ErrorHandler, Location};

impl DatastoreWrapper {
    pub async fn device_status(
        &self,
        device_id: String,
        token: &str,
    ) -> Result<DeviceStatus, Error> {
        let request = GetByIdRequest {
            params: Some(Params {
                id: device_id,
                table: String::from("devices"),
            }),
            query: Some(Query {
                pluck: String::from("status"),
                durability: String::from("soft"),
            }),
        };

        let response = self.inner.get_by_id(request, token).await?;

        let status = Self::internal_ds_parse_response_data(&response.data)?;

        Ok(map_status_value_to_enum(status))
    }

    fn internal_ds_parse_response_data(data: &str) -> Result<String, Error> {
        serde_json::from_str::<serde_json::Value>(data)
            .handle_err(location!())?
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|obj| obj.as_object())
            .and_then(|map| map.get("status"))
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
            .ok_or("Failed to parse response")
            .handle_err(location!())
    }
}
