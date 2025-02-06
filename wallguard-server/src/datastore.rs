use crate::{parser::parsed_message::ParsedMessage, utils::digest};
use chrono::Utc;
use libfireparse::Configuration as ClientConfiguration;
use nullnet_libdatastore::{
    BatchCreateBody, BatchCreateRequest, CreateParams, CreateRequest, DatastoreClient,
    DatastoreConfig, Error as DSError, ErrorKind as DSErrorKind, LoginBody, LoginData,
    LoginRequest, Params, Query, Response as DSResponse, UpdateRequest,
};
use serde_json::json;
use std::str::FromStr;
use tonic::{metadata::MetadataValue, Request};

#[derive(Debug, Clone)]
pub struct DatastoreWrapper {
    inner: DatastoreClient,
}

impl DatastoreWrapper {
    pub fn new() -> Self {
        let config = DatastoreConfig::from_env();
        let inner = DatastoreClient::new(config);
        Self { inner }
    }

    pub fn set_token_for_request<T>(request: &mut Request<T>, token: &str) -> Result<(), DSError> {
        let value = MetadataValue::from_str(token).map_err(|e| DSError {
            kind: DSErrorKind::ErrorRequestFailed,
            message: e.to_string(),
        })?;

        request.metadata_mut().insert("authorization", value);

        Ok(())
    }

    pub async fn login(
        &self,
        account_id: String,
        account_secret: String,
    ) -> Result<String, DSError> {
        let request = Request::new(LoginRequest {
            body: Some(LoginBody {
                data: Some(LoginData {
                    account_id,
                    account_secret,
                }),
            }),
        });

        let response = self.inner.login(request).await?;

        Ok(response.token)
    }

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

    pub async fn packets_insert(
        &self,
        token: &str,
        parsed_message: ParsedMessage,
    ) -> Result<DSResponse, DSError> {
        let records = serde_json::to_string(&parsed_message).map_err(|e| DSError {
            kind: DSErrorKind::ErrorRequestFailed,
            message: e.to_string(),
        })?;

        let mut request = Request::new(BatchCreateRequest {
            params: Some(CreateParams {
                table: String::from("packets"),
            }),
            query: Some(Query {
                pluck: String::new(),
            }),
            body: Some(BatchCreateBody {
                records,
                entity_prefix: String::from("PK"),
            }),
        });

        Self::set_token_for_request(&mut request, token)?;

        let response = self.inner.batch_create(request).await?;

        Ok(response)
    }

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

    pub async fn config_upload(
        &self,
        token: &str,
        device_id: String,
        config: ClientConfiguration,
        status: String,
    ) -> Result<DSResponse, DSError> {
        // 1. Create new config record
        // 2. Insert related aliases
        // 3. Insert related rules

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
                "entity_prefix": String::from("CFG")
            })
            .to_string(),
        });

        Self::set_token_for_request(&mut request, token)?;

        let response = self.inner.create(request).await?;

        if !response.success {
            return Err(DSError {
                kind: DSErrorKind::ErrorRequestFailed,
                message: format!(
                    "Failed to create configuration record: {}",
                    response.message
                ),
            });
        }

        let rjson: serde_json::Value =
            serde_json::from_str(&response.data).map_err(|e| DSError {
                kind: DSErrorKind::ErrorRequestFailed,
                message: format!("Could not parse DS response: {}", e),
            })?;

        let config_id = rjson
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|obj| obj.as_object())
            .and_then(|map| map.get("id"))
            .and_then(|v| v.as_str())
            .map(|id| id.to_string())
            .ok_or(DSError {
                kind: DSErrorKind::ErrorRequestFailed,
                message: String::from("Failed to parse DS response. Either the format is unexpected or the configuration id is missing"),
            })?;

        let rules_with_id: Vec<serde_json::Value> = config
            .rules
            .into_iter()
            .map(|rule| {
                let mut json = serde_json::to_value(rule).expect("Rule serialization failed");
                json["device_configuration_id"] = json!(config_id.clone());
                json["device_rules_status"] = json!(status.clone());
                json
            })
            .collect();

        let mut request = Request::new(BatchCreateRequest {
            params: Some(CreateParams {
                table: String::from("device_rules"),
            }),
            query: Some(Query {
                pluck: String::new(),
            }),
            body: Some(BatchCreateBody {
                records: serde_json::to_string(&serde_json::Value::Array(rules_with_id)).unwrap(),
                entity_prefix: String::from("RL"),
            }),
        });

        Self::set_token_for_request(&mut request, token)?;

        let response = self.inner.batch_create(request).await?;

        if !response.success {
            return Err(DSError {
                kind: DSErrorKind::ErrorRequestFailed,
                message: format!("Failed to create rules records: {}", response.message),
            });
        }

        let aliases_with_id: Vec<serde_json::Value> = config
            .aliases
            .into_iter()
            .map(|rule| {
                let mut json = serde_json::to_value(rule).expect("Rule serialization failed");
                json["device_configuration_id"] = json!(config_id.clone());
                json["device_alias_status"] = json!(status.clone());
                json
            })
            .collect();

        let mut request = Request::new(BatchCreateRequest {
            params: Some(CreateParams {
                table: String::from("device_aliases"),
            }),
            query: Some(Query {
                pluck: String::new(),
            }),
            body: Some(BatchCreateBody {
                records: serde_json::to_string(&serde_json::Value::Array(aliases_with_id)).unwrap(),
                entity_prefix: String::from("AL"),
            }),
        });

        Self::set_token_for_request(&mut request, token)?;

        let response = self.inner.batch_create(request).await?;

        if !response.success {
            return Err(DSError {
                kind: DSErrorKind::ErrorRequestFailed,
                message: format!("Failed to create aliases records: {}", response.message),
            });
        }

        Ok(response)
    }
}
