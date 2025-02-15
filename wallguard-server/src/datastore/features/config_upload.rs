use std::{collections::HashMap, i32};

use futures::future::join_all;
use serde_json::json;
use tonic::Request;

use crate::{datastore::DatastoreWrapper, utils::digest};
use libfireparse::Configuration as ClientConfiguration;
use nullnet_libdatastore::{
    AdvanceFilter, BatchCreateBody, BatchCreateRequest, CreateParams, CreateRequest,
    Error as DSError, ErrorKind as DSErrorKind, GetByFilterBody, GetByFilterRequest, Params, Query,
    Response as DSResponse, UpdateRequest,
};

/*
!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
!!!! ABANDON HOPE EVERYONE WHO ENTERS HERE !!!!!
!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
*/

impl DatastoreWrapper {
    pub async fn config_upload(
        &self,
        token: &str,
        device_id: String,
        config: ClientConfiguration,
        status: String,
    ) -> Result<String, DSError> {
        let prev_info = self
            .internal_cu_fetch_latest_config_info(&device_id, token)
            .await;

        if prev_info.is_err() {
            // No previous versions found
            return self
                .internal_cu_parse_and_insert_new_config(token, device_id, config, status)
                .await;
        } else {
            let (prev_digest, config_id, last_version) = prev_info.unwrap();
            let new_digest = digest(&config.raw_content);

            if prev_digest == new_digest && false {
                let (r1, r2, r3) = tokio::join!(
                    self.internal_cu_update_configuration_version(
                        &config_id,
                        last_version + 1,
                        &token
                    ),
                    self.internal_cu_update_related_records(
                        &config_id,
                        token,
                        "device_aliases",
                        "device_alias_status",
                        &status,
                    ),
                    self.internal_cu_update_related_records(
                        &config_id,
                        token,
                        "device_rules",
                        "device_rule_status",
                        &status,
                    )
                );

                if r1.is_err() || r2.is_err() || r3.is_err() {
                    return Err(DSError {
                        kind: DSErrorKind::ErrorRequestFailed,
                        message: String::from("Configuration update failed"),
                    });
                }
                return Ok(String::from("Updated existing configuration record"));
            } else {
                // Digests do not match, insert new configuration
                return self
                    .internal_cu_parse_and_insert_new_config(token, device_id, config, status)
                    .await;
            }
        }
    }

    async fn internal_cu_parse_and_insert_new_config(
        &self,
        token: &str,
        device_id: String,
        config: ClientConfiguration,
        status: String,
    ) -> Result<String, DSError> {
        let config_id = self
            .internal_cu_create_configuration(token, device_id, &config)
            .await?;

        let (r1, r2, r3) = tokio::join!(
            self.internal_cu_insert_related_records(
                token,
                "device_rules",
                "RL",
                &config.rules,
                &config_id,
                "device_rule_status",
                Some(&status),
            ),
            self.internal_cu_insert_related_records(
                token,
                "device_aliases",
                "AL",
                &config.aliases,
                &config_id,
                "device_alias_status",
                Some(&status),
            ),
            self.internal_cu_insert_related_records(
                token,
                "device_interfaces",
                "IF",
                &config.interfaces,
                &config_id,
                "",
                None,
            )
        );

        r1?;
        r2?;
        r3?;

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
                durability: String::from("soft"),
            }),
            body: json!({
                "device_id": device_id,
                "raw_content": config.raw_content,
                "digest": digest(&config.raw_content),
                "hostname": config.hostname,
                "config_version": 1,
                "entity_prefix": "CFG"
            })
            .to_string(),
        });

        Self::set_token_for_request(&mut request, token)?;

        let response = self.inner.create(request).await?;
        self.internal_cu_extract_id_from_response(&response, "configuration") // Extracts the ID from the response
    }

    /// Extracts the ID from a datastore response.
    fn internal_cu_extract_id_from_response(
        &self,
        response: &DSResponse,
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
                message: format!("Could not parse DS response: {e}"),
            })?;

        rjson.as_array()
            .and_then(|arr| arr.first())
            .and_then(|obj| obj.as_object())
            .and_then(|map| map.get("id"))
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
            .ok_or(DSError {
                kind: DSErrorKind::ErrorRequestFailed,
                message: format!(
                    "Failed to parse DS response. Either the format is unexpected or the {record_type} ID is missing",
                ),
            })
    }

    /// Inserts related records (rules/aliases) into the datastore.
    #[allow(clippy::too_many_arguments)]
    async fn internal_cu_insert_related_records<T: serde::Serialize>(
        &self,
        token: &str,
        table: &str,
        entity_prefix: &str,
        records: &[T],
        config_id: &str,
        status_field: &str,
        status_value: Option<&str>,
    ) -> Result<(), DSError> {
        if records.is_empty() {
            return Ok(());
        }

        let records_with_id: Vec<serde_json::Value> = records
            .iter()
            .map(|record| {
                let mut json = serde_json::to_value(record).expect("Serialization failed");
                json["device_configuration_id"] = json!(config_id);

                if status_value.is_some() {
                    json[status_field] = json!(status_value.unwrap());
                }

                json
            })
            .collect();

        let mut request = Request::new(BatchCreateRequest {
            params: Some(CreateParams {
                table: table.to_string(),
            }),
            query: Some(Query {
                pluck: String::new(),
                durability: String::from("soft"),
            }),
            body: Some(BatchCreateBody {
                records: serde_json::to_string(&serde_json::Value::Array(records_with_id)).unwrap(),
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

    async fn internal_cu_fetch_latest_config_info(
        &self,
        device_id: &str,
        token: &str,
    ) -> Result<(String, String, i64), ()> {
        let mut request = Request::new(GetByFilterRequest {
            body: Some(GetByFilterBody {
                pluck: vec![
                    String::from("id"),
                    String::from("digest"),
                    String::from("config_version"),
                ],
                advance_filters: vec![AdvanceFilter {
                    field: String::from("device_id"),
                    values: format!("[\"{}\"]", device_id),
                    r#type: String::from("criteria"),
                    operator: String::from("equal"),
                    entity: String::from("device_configurations"),
                }],
                order_by: String::from("timestamp"),
                order_direction: String::from("desc"),
                limit: 1,
                offset: 0,
                joins: vec![],
                multiple_sort: vec![],
                pluck_object: HashMap::new(),
                date_format: String::new(),
            }),
            params: Some(Params {
                table: String::from("device_configurations"),
                id: String::new(),
            }),
        });

        Self::set_token_for_request(&mut request, token).map_err(|_| ())?;

        let response = self.inner.get_by_filter(request).await.map_err(|_| ())?;

        if !response.success {
            return Err(());
        }

        let json: serde_json::Value = serde_json::from_str(&response.data).map_err(|_| ())?;

        let object = json
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|e| e.as_object())
            .ok_or(())?;

        let digest = object
            .get("digest")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
            .ok_or(())?;

        let id = object
            .get("id")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
            .ok_or(())?;

        let version = object
            .get("config_version")
            .and_then(|v| v.as_i64())
            .ok_or(())?;

        Ok((digest, id, version))
    }

    async fn internal_cu_update_configuration_version(
        &self,
        config_id: &str,
        new_version: i64,
        token: &str,
    ) -> Result<(), String> {
        let mut request = Request::new(UpdateRequest {
            params: Some(Params {
                id: config_id.to_string(),
                table: String::from("device_configurations"),
            }),
            query: Some(Query {
                pluck: String::from(""),
                durability: String::from("soft"),
            }),
            body: json!({
                "config_version": new_version,
            })
            .to_string(),
        });

        Self::set_token_for_request(&mut request, token).map_err(|e| e.message)?;

        let response = self.inner.update(request).await.map_err(|e| e.message)?;

        if !response.success {
            return Err(format!("{} !!!ggg!! {}", response.message, response.error));
        } else {
            return Ok(());
        }
    }

    async fn internal_cu_update_related_records(
        &self,
        config_id: &str,
        token: &str,
        table: &str,
        status_field: &str,
        status: &str,
    ) -> Result<(), String> {
        //!!!!!WARNING!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
        //! THIS METHOD FETCHES THE RELATED RECORDS AND UPDATES THEM ONE BY ONE                               !!!!!
        //! CURRENT DATASTORE API DOES NOT PROVIDE BATCH UPDATES, THIS IS WHY IT IS IMPLEMENTED THE WAY IT IS !!!!!
        //! IT MUST BE RE-WRITTEN LATER                                                                       !!!!!
        //!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

        let mut request = Request::new(GetByFilterRequest {
            body: Some(GetByFilterBody {
                pluck: vec![String::from("id")],
                advance_filters: vec![AdvanceFilter {
                    field: String::from("device_configuration_id"),
                    values: format!("[\"{}\"]", config_id),
                    r#type: String::from("criteria"),
                    operator: String::from("equal"),
                    entity: String::from(table),
                }],
                order_by: String::from("timestamp"),
                order_direction: String::from("desc"),
                limit: i32::MAX,
                offset: 0,
                joins: vec![],
                multiple_sort: vec![],
                pluck_object: HashMap::new(),
                date_format: String::new(),
            }),
            params: Some(Params {
                table: String::from(table),
                id: String::new(),
            }),
        });

        Self::set_token_for_request(&mut request, token).map_err(|e| e.message)?;

        let records_response = self
            .inner
            .get_by_filter(request)
            .await
            .map_err(|e| e.message)?;

        if !records_response.success {
            return Err(format!(
                "{} <-----> {}",
                records_response.message, records_response.error
            ));
        }

        let records_json: serde_json::Value =
            serde_json::from_str(&records_response.data).map_err(|e| e.to_string())?;

        let ids: Vec<String> = records_json
            .as_array()
            .ok_or("Failed to parse records JSON")?
            .iter()
            .filter_map(|item| item.get("id")?.as_str().map(|s| s.to_string()))
            .collect();

        let futures: Vec<_> = ids
            .into_iter()
            .map(|id| {
                let mut body = json!({});
                body[status_field] = json!(status);

                let mut request1 = Request::new(UpdateRequest {
                    params: Some(Params {
                        id,
                        table: table.to_string(),
                    }),
                    query: Some(Query {
                        pluck: String::new(),
                        durability: String::from("soft"),
                    }),
                    body: body.to_string(),
                });

                let set_token_result = Self::set_token_for_request(&mut request1, token);
                async move {
                    if let Err(e) = set_token_result {
                        return Err(e.message);
                    }
                    let _ = self.inner.update(request1).await;
                    Ok(())
                }
            })
            .collect();

        for r in join_all(futures).await {
            r?;
        }

        Ok(())
    }
}
