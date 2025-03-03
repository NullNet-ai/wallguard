mod latest_config_info;
mod utils;

use super::utils::convert_status;
use crate::{datastore::DatastoreWrapper, proto::wallguard::ConfigStatus, utils::digest};
use latest_config_info::LatestConfigInfo;
use libfireparse::Configuration as ClientConfiguration;
use nullnet_libdatastore::{
    AdvanceFilter, BatchCreateBody, BatchCreateRequest, BatchUpdateBody, BatchUpdateRequest,
    CreateBody, CreateParams, CreateRequest, DatastoreClient, GetByFilterBody, GetByFilterRequest,
    Params, Query, UpdateRequest,
};
use nullnet_liberror::Error;
use serde_json::json;
use std::collections::HashMap;
use utils::parse_configuraion_id;

impl DatastoreWrapper {
    pub async fn config_upload(
        &self,
        token: &str,
        device_id: String,
        config: ClientConfiguration,
        status: ConfigStatus,
    ) -> Result<String, Error> {
        let status = convert_status(status);

        let prev_info_result =
            Self::internal_cu_fetch_latest_config_info(self.inner.clone(), &device_id, token).await;

        if prev_info_result.is_err() {
            // No previous versions found
            return Self::internal_cu_parse_and_insert_new_config(
                self.inner.clone(),
                token,
                device_id,
                config,
                status,
            )
            .await;
        } else {
            let prev_info = prev_info_result.unwrap();
            let new_digest = digest(&config.raw_content);

            if prev_info.digest == new_digest {
                let (r1, r2, r3) = tokio::join!(
                    Self::internal_cu_update_configuration_version(
                        self.inner.clone(),
                        &prev_info.id,
                        prev_info.version + 1,
                        token
                    ),
                    Self::internal_cu_update_related_records(
                        self.inner.clone(),
                        &prev_info.id,
                        token,
                        "device_aliases",
                        "device_alias_status",
                        &status,
                    ),
                    Self::internal_cu_update_related_records(
                        self.inner.clone(),
                        &prev_info.id,
                        token,
                        "device_rules",
                        "device_rule_status",
                        &status,
                    )
                );

                r1?;
                r2?;
                r3?;

                Ok(prev_info.id)
            } else {
                // Digests do not match, insert new configuration
                Self::internal_cu_parse_and_insert_new_config(
                    self.inner.clone(),
                    token,
                    device_id,
                    config,
                    status,
                )
                .await
            }
        }
    }

    async fn internal_cu_fetch_latest_config_info(
        mut client: DatastoreClient,
        device_id: &str,
        token: &str,
    ) -> Result<LatestConfigInfo, Error> {
        let request = GetByFilterRequest {
            body: Some(GetByFilterBody {
                pluck: vec![
                    String::from("id"),
                    String::from("digest"),
                    String::from("config_version"),
                ],
                advance_filters: vec![AdvanceFilter {
                    field: String::from("device_id"),
                    values: format!("[\"{device_id}\"]"),
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
        };

        let response = client.get_by_filter(request, token).await?;
        LatestConfigInfo::from_response_data(&response)
    }

    async fn internal_cu_parse_and_insert_new_config(
        client: DatastoreClient,
        token: &str,
        device_id: String,
        config: ClientConfiguration,
        status: String,
    ) -> Result<String, Error> {
        let config_id =
            Self::internal_cu_create_configuration(client.clone(), token, device_id, &config)
                .await?;

        let (r1, r2, r3) = tokio::join!(
            Self::internal_cu_insert_related_records(
                client.clone(),
                token,
                "device_rules",
                "RL",
                &config.rules,
                &config_id,
                "device_rule_status",
                Some(&status),
            ),
            Self::internal_cu_insert_related_records(
                client.clone(),
                token,
                "device_aliases",
                "AL",
                &config.aliases,
                &config_id,
                "device_alias_status",
                Some(&status),
            ),
            Self::internal_cu_insert_related_records(
                client.clone(),
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
        mut client: DatastoreClient,
        token: &str,
        device_id: String,
        config: &ClientConfiguration,
    ) -> Result<String, Error> {
        let request = CreateRequest {
            params: Some(CreateParams {
                table: String::from("device_configurations"),
            }),
            query: Some(Query {
                pluck: String::from("id"),
                durability: String::from("hard"),
            }),
            body: Some(CreateBody {
                record: json!({
                    "device_id": device_id,
                    "raw_content": config.raw_content,
                    "digest": digest(&config.raw_content),
                    "hostname": config.hostname,
                    "config_version": 1,
                })
                .to_string(),
                entity_prefix: String::from("CFG"),
            }),
        };

        let response = client.create(request, token).await?;
        parse_configuraion_id(&response)
    }

    /// Inserts related records (rules/aliases) into the datastore.
    #[allow(clippy::too_many_arguments)]
    async fn internal_cu_insert_related_records<T: serde::Serialize>(
        mut client: DatastoreClient,
        token: &str,
        table: &str,
        entity_prefix: &str,
        records: &[T],
        config_id: &str,
        status_field: &str,
        status_value: Option<&str>,
    ) -> Result<(), Error> {
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

        let request = BatchCreateRequest {
            params: Some(CreateParams {
                table: table.to_string(),
            }),
            query: Some(Query {
                pluck: String::new(),
                durability: String::from("hard"),
            }),
            body: Some(BatchCreateBody {
                records: serde_json::to_string(&serde_json::Value::Array(records_with_id)).unwrap(),
                entity_prefix: entity_prefix.to_string(),
            }),
        };

        let _ = client.batch_create(request, token).await?;

        Ok(())
    }

    async fn internal_cu_update_configuration_version(
        mut client: DatastoreClient,
        config_id: &str,
        new_version: i64,
        token: &str,
    ) -> Result<(), Error> {
        let request = UpdateRequest {
            params: Some(Params {
                id: config_id.to_string(),
                table: String::from("device_configurations"),
            }),
            query: Some(Query {
                pluck: String::new(),
                durability: String::from("hard"),
            }),
            body: json!({
                "config_version": new_version,
            })
            .to_string(),
        };

        let _ = client.update(request, token).await?;

        Ok(())
    }

    async fn internal_cu_update_related_records(
        mut client: DatastoreClient,
        config_id: &str,
        token: &str,
        table: &str,
        status_field: &str,
        status: &str,
    ) -> Result<(), Error> {
        let mut body = json!({});
        body[status_field] = json!(status);

        let request = BatchUpdateRequest {
            params: Some(Params {
                id: String::new(),
                table: String::from(table),
            }),
            body: Some(BatchUpdateBody {
                advance_filters: vec![AdvanceFilter {
                    field: String::from("device_configuration_id"),
                    values: format!("[\"{config_id}\"]"),
                    r#type: String::from("criteria"),
                    operator: String::from("equal"),
                    entity: String::from(table),
                }],
                updates: body.to_string(),
            }),
        };

        let _ = client.batch_update(request, token).await?;

        Ok(())
    }
}
