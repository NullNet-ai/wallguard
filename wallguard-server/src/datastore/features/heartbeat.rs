use chrono::Utc;
use serde_json::json;
use tonic::Request;

use crate::datastore::DatastoreWrapper;
use nullnet_libdatastore::{CreateParams, CreateRequest, GetByIdRequest, Params, Query};

impl DatastoreWrapper {
    pub async fn heartbeat(
        &self,
        token: &str,
        device_id: String,
    ) -> Result<(String, bool, bool), String> {
        let mut request = Request::new(CreateRequest {
            params: Some(CreateParams {
                table: String::from("device_heartbeats"),
            }),
            query: Some(Query {
                pluck: String::new(),
                durability: String::from("hard"),
            }),
            body: json!({
                "device_id": device_id.clone(),
                "timestamp": Utc::now().to_rfc3339(),
                "entity_prefix": String::from("HB")
            })
            .to_string(),
        });

        Self::set_token_for_request(&mut request, token).map_err(|e| e.to_string())?;

        let response = self
            .inner
            .create(request)
            .await
            .map_err(|e| format!("Request failed: {e}"))?;

        if !response.success {
            return Err(format!("Failed to save heartbeat: {}", response.message));
        }

        let (status, is_remote_access_enabled, is_monitoring_enabled) =
            self.fetch_device_info(device_id, token).await?;

        Ok((status, is_remote_access_enabled, is_monitoring_enabled))
    }

    async fn fetch_device_info(
        &self,
        device_id: String,
        token: &str,
    ) -> Result<(String, bool, bool), String> {
        let mut request = Request::new(GetByIdRequest {
            params: Some(Params {
                id: device_id,
                table: String::from("devices"),
            }),
            query: Some(Query {
                pluck: String::from("status,is_monitoring_enabled,is_remote_access_enabled"),
                durability: String::from("hard"),
            }),
        });

        Self::set_token_for_request(&mut request, token).map_err(|e| e.to_string())?;

        let response = self
            .inner
            .get_by_id(request)
            .await
            .map_err(|e| format!("Request failed: {e}"))?;

        let (status, is_remote_access_enabled, is_monitoring_enabled) =
            Self::internal_hb_parse_response_data(&response.data)?;

        Ok((status, is_remote_access_enabled, is_monitoring_enabled))
    }

    fn internal_hb_parse_response_data(data: &str) -> Result<(String, bool, bool), String> {
        let json = serde_json::from_str::<serde_json::Value>(data)
            .map_err(|e| format!("Could not parse DS response: {e}"))?;

        let object = json
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|obj| obj.as_object())
            .ok_or(String::from("Failed to parse DS response."))?;

        let is_monitoring_enabled = object
            .get("is_monitoring_enabled")
            .and_then(serde_json::Value::as_bool)
            .ok_or(String::from(
                "Failed to parse DS response: could not parse 'is_monitoring_enabled'",
            ))?;

        let is_remote_access_enabled = object
            .get("is_remote_access_enabled")
            .and_then(serde_json::Value::as_bool)
            .ok_or(String::from(
                "Failed to parse DS response: could not parse 'is_remote_access_enabled'",
            ))?;

        let status = object
            .get("status")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
            .ok_or(String::from(
                "Failed to parse DS response: could not parse 'status'",
            ))?;

        Ok((status, is_remote_access_enabled, is_monitoring_enabled))
    }
}
