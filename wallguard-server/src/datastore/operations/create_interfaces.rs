use crate::datastore::{Datastore, builders::BatchCreateRequestBuilder, db_tables::DBTable};
use crate::fireparse::NetworkInterface;
use nullnet_libdatastore::ResponseData;
use nullnet_liberror::Error;
use serde_json::json;
use std::collections::HashMap;

impl Datastore {
    pub async fn create_interfaces(
        &self,
        token: &str,
        interfaces: &Vec<NetworkInterface>,
        config_id: &str,
    ) -> Result<(), Error> {
        if interfaces.is_empty() {
            return Ok(());
        }

        let records: Vec<serde_json::Value> = interfaces
            .iter()
            .map(|iface| {
                let mut json = serde_json::to_value(iface).unwrap_or_default();
                json["device_configuration_id"] = json!(config_id);
                json
            })
            .collect();

        let request = BatchCreateRequestBuilder::new()
            .table(DBTable::DeviceInterfaces)
            .durability("hard")
            .entity_prefix("IF")
            .records(serde_json::to_string(&serde_json::Value::Array(records)).unwrap())
            .build();

        let response_data = self.inner.clone().batch_create(request, token).await?;
        let result = InterfaceInsertionResult::from_response_data(response_data);

        let mut records: Vec<serde_json::Value> = vec![];

        for interface in interfaces {
            for address in &interface.addresses {
                if let Some(id) = result.get_id_by_device(&interface.device) {
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

        let request = BatchCreateRequestBuilder::new()
            .table(DBTable::DeviceInterfaceAddresses)
            .durability("hard")
            .records(serde_json::to_string(&serde_json::Value::Array(records)).unwrap())
            .entity_prefix("AD")
            .build();

        let _ = self.inner.clone().batch_create(request, token).await?;

        Ok(())
    }
}

pub struct InterfaceInsertionResult {
    map: HashMap<String, String>,
}

impl InterfaceInsertionResult {
    pub fn from_response_data(data: ResponseData) -> Self {
        let json: serde_json::Value =
            serde_json::from_str(&data.data).expect("Failed to parse response data");

        let mut map = HashMap::new();

        if let Some(array) = json.as_array() {
            for value in array {
                if let Some(object) = value.as_object() {
                    let Some(id) = object.get("id").and_then(|obj| obj.as_str()) else {
                        continue;
                    };

                    let Some(device) = object.get("device").and_then(|obj| obj.as_str()) else {
                        continue;
                    };

                    map.insert(String::from(device), String::from(id));
                }
            }
        }

        Self { map }
    }

    pub fn get_id_by_device(&self, device: &str) -> Option<&String> {
        self.map.get(device)
    }
}
