use serde_json::json;
use tonic::Request;

use crate::{datastore::DatastoreWrapper, utils::digest};
use libfireparse::Configuration as ClientConfiguration;
use nullnet_libdatastore::{
    BatchCreateBody, BatchCreateRequest, CreateParams, CreateRequest, Error as DSError,
    ErrorKind as DSErrorKind, Query, Response as DSResponse,
};

impl DatastoreWrapper {
    pub async fn config_upload(
        &self,
        token: &str,
        device_id: String,
        config: ClientConfiguration,
        status: String,
    ) -> Result<String, DSError> {
        let config_id = self
            .internal_cu_create_configuration(token, device_id, &config)
            .await?;

        self.internal_cu_insert_related_records(
            token,
            "device_rules",
            "RL",
            &config.rules,
            &config_id,
            "device_rule_status",
            &status,
        )
        .await?;

        self.internal_cu_insert_related_records(
            token,
            "device_aliases",
            "AL",
            &config.aliases,
            &config_id,
            "device_alias_status",
            &status,
        )
        .await?;

        Ok(config_id)
    }

    /// Creates a new configuration record and returns the generated config ID.
    async fn internal_cu_create_configuration(
        &self,
        token: &str,
        device_id: String,
        config: &ClientConfiguration,
    ) -> Result<String, DSError> {
        let mut request = Request::new(CreateRequest {
            params: Some(CreateParams {
                table: String::from("device_configurations"),
            }),
            query: Some(Query {
                pluck: String::from("id"),
            }),
            body: json!({
                "device_id": device_id,
                "raw_content": config.raw_content,
                "digest": digest(&config.raw_content),
                // @TODO: Temporary fix, needs to be removed
                "entity_prefix": "CFG"
            })
            .to_string(),
        });

        Self::set_token_for_request(&mut request, token)?;

        let response = self.inner.create(request).await?;
        self.internal_cu_extract_id_from_response(response, "configuration") // Extracts the ID from the response
    }

    /// Extracts the ID from a datastore response.
    fn internal_cu_extract_id_from_response(
        &self,
        response: DSResponse,
        record_type: &str,
    ) -> Result<String, DSError> {
        if !response.success {
            return Err(DSError {
                kind: DSErrorKind::ErrorRequestFailed,
                message: format!(
                    "Failed to create {} record: {}",
                    record_type, response.message
                ),
            });
        }

        let rjson: serde_json::Value =
            serde_json::from_str(&response.data).map_err(|e| DSError {
                kind: DSErrorKind::ErrorRequestFailed,
                message: format!("Could not parse DS response: {}", e),
            })?;

        rjson.as_array()
            .and_then(|arr| arr.first())
            .and_then(|obj| obj.as_object())
            .and_then(|map| map.get("id"))
            .and_then(|v| v.as_str())
            .map(|id| id.to_string())
            .ok_or(DSError {
                kind: DSErrorKind::ErrorRequestFailed,
                message: format!(
                    "Failed to parse DS response. Either the format is unexpected or the {} ID is missing",
                    record_type
                ),
            })
    }

    /// Inserts related records (rules/aliases) into the datastore.
    async fn internal_cu_insert_related_records<T: serde::Serialize>(
        &self,
        token: &str,
        table: &str,
        entity_prefix: &str,
        records: &[T],
        config_id: &str,
        status_field: &str,
        status_value: &str,
    ) -> Result<(), DSError> {
        let records_with_id: Vec<serde_json::Value> = records
            .iter()
            .map(|record| {
                let mut json = serde_json::to_value(record).expect("Serialization failed");
                json["device_configuration_id"] = json!(config_id);
                json[status_field] = json!(status_value);
                json
            })
            .collect();

        let mut request = Request::new(BatchCreateRequest {
            params: Some(CreateParams {
                table: table.to_string(),
            }),
            query: Some(Query {
                pluck: String::new(),
            }),
            body: Some(BatchCreateBody {
                records: serde_json::to_string(&serde_json::Value::Array(records_with_id)).unwrap(),
                // @TODO: Temporary fix, needs to be removed
                entity_prefix: entity_prefix.to_string(),
            }),
        });

        Self::set_token_for_request(&mut request, token)?;

        let response = self.inner.batch_create(request).await?;

        if !response.success {
            return Err(DSError {
                kind: DSErrorKind::ErrorRequestFailed,
                message: format!("Failed to create {} records: {}", table, response.message),
            });
        }

        Ok(())
    }
}
