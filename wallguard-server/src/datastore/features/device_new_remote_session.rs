use crate::{datastore::DatastoreWrapper, tunnel::RAType};
use nullnet_libdatastore::{
    CreateBody, CreateParams, CreateRequest, GetByIdRequest, Params, Query,
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use serde_json::json;

impl DatastoreWrapper {
    pub async fn device_new_remote_session(
        &self,
        token: &str,
        device_id: String,
        remote_access_type: RAType,
        remote_access_session: String,
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
                    "remote_access_session": remote_access_session,
                    "remote_access_status": "active",
                    "categories": vec![remote_access_type.to_string()]
                })
                .to_string(),
                entity_prefix: String::from("RAS"),
            }),
        };

        let _ = self.inner.clone().create(request, token).await?;

        Ok(())
    }

    pub async fn device_check_if_remote_access_enabled(
        &self,
        token: &str,
        device_id: String,
    ) -> Result<bool, Error> {
        let request = GetByIdRequest {
            params: Some(Params {
                id: device_id,
                table: String::from("devices"),
            }),
            query: Some(Query {
                pluck: String::from("is_remote_access_enabled"),
                durability: String::from("hard"),
            }),
        };

        let response = self.inner.clone().get_by_id(request, token).await?;
        let enabled = Self::internal_ra_parse_response_data(&response.data)?;

        Ok(enabled)
    }

    fn internal_ra_parse_response_data(data: &str) -> Result<bool, Error> {
        serde_json::from_str::<serde_json::Value>(data)
            .handle_err(location!())?
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|obj| obj.as_object())
            .and_then(|map| map.get("is_remote_access_enabled"))
            .and_then(|v| v.as_bool())
            .ok_or("Failed to parse response")
            .handle_err(location!())
    }
}
