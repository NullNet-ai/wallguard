use super::insert_iface_result::InterfaceInsertionResult;
use crate::datastore::DatastoreWrapper;
use libfireparse::NetworkInterface;
use nullnet_libdatastore::{
    BatchCreateBody, BatchCreateRequest, CreateParams, DatastoreClient, Query,
};
use nullnet_liberror::Error;
use serde_json::json;

impl DatastoreWrapper {
    pub(crate) async fn internal_cu_insert_interfaces(
        client: DatastoreClient,
        token: &str,
        interfaces: &Vec<NetworkInterface>,
        config_id: &str,
    ) -> Result<(), Error> {
        if interfaces.is_empty() {
            return Ok(());
        }

        let if_result = Self::internal_cu_insert_interfaces_records(
            client.clone(),
            token,
            interfaces,
            config_id,
        )
        .await?;

        Self::internal_cu_insert_addresses_records(client, token, interfaces, if_result).await?;

        Ok(())
    }

    async fn internal_cu_insert_interfaces_records(
        mut client: DatastoreClient,
        token: &str,
        interfaces: &[NetworkInterface],
        config_id: &str,
    ) -> Result<InterfaceInsertionResult, Error> {
        let records: Vec<serde_json::Value> = interfaces
            .iter()
            .map(|iface| {
                let mut json = json!({});
                json["device_configuration_id"] = json!(config_id);
                json["name"] = json!(iface.name);
                json["device"] = json!(iface.device);
                json
            })
            .collect();

        let request = BatchCreateRequest {
            params: Some(CreateParams {
                table: String::from("device_interfaces"),
            }),
            query: Some(Query {
                pluck: String::from("id,device"),
                durability: String::from("hard"),
            }),
            body: Some(BatchCreateBody {
                records: serde_json::to_string(&serde_json::Value::Array(records)).unwrap(),
                entity_prefix: String::from("IF"),
            }),
        };

        let response_data = client.batch_create(request, token).await?;
        Ok(InterfaceInsertionResult::from_response_data(response_data))
    }

    async fn internal_cu_insert_addresses_records(
        mut client: DatastoreClient,
        token: &str,
        interfaces: &Vec<NetworkInterface>,
        if_result: InterfaceInsertionResult,
    ) -> Result<(), Error> {
        let mut records: Vec<serde_json::Value> = vec![];
        for interface in interfaces {
            for address in &interface.addresses {
                if let Some(id) = if_result.get_id_by_device(&interface.device) {
                    let mut json = json!({});

                    json["device_interface_id"] = json!(id);
                    json["address"] = json!(address.address);
                    json["version"] = json!(address.version);

                    records.push(json);
                }
            }
        }

        if records.is_empty() {
            return Ok(());
        }

        let request = BatchCreateRequest {
            params: Some(CreateParams {
                table: String::from("device_addresses"),
            }),
            query: Some(Query {
                pluck: String::new(),
                durability: String::from("hard"),
            }),
            body: Some(BatchCreateBody {
                records: serde_json::to_string(&serde_json::Value::Array(records)).unwrap(),
                entity_prefix: String::from("AD"),
            }),
        };

        let _ = client.batch_create(request, token).await?;

        Ok(())
    }
}
