use crate::{datastore::DatastoreWrapper, ssh_keypair::SSHKeypair};
use nullnet_libdatastore::{AdvanceFilter, GetByFilterBody, GetByFilterRequest, Params};
use std::collections::HashMap;

impl DatastoreWrapper {
    pub async fn fetch_ssh_keypair_for_device(
        &self,
        device_id: &str,
        token: &str,
    ) -> Option<SSHKeypair> {
        let request = GetByFilterRequest {
            body: Some(GetByFilterBody {
                pluck: vec![
                    String::from("public_key"),
                    String::from("private_key"),
                    String::from("passphrase"),
                ],
                advance_filters: vec![AdvanceFilter {
                    r#type: String::from("criteria"),
                    field: String::from("device_id"),
                    operator: String::from("equal"),
                    entity: String::from("device_ssh_keys"),
                    values: format!("[\"{device_id}\"]"),
                }],
                order_by: String::new(),
                limit: 1,
                offset: 0,
                order_direction: String::new(),
                joins: vec![],
                multiple_sort: vec![],
                pluck_object: HashMap::default(),
                date_format: String::new(),
            }),
            params: Some(Params {
                id: String::new(),
                table: String::from("device_ssh_keys"),
            }),
        };

        let response = self
            .inner
            .clone()
            .get_by_filter(request, token)
            .await
            .ok()?;

        let json: serde_json::Value = serde_json::from_str(&response.data).ok()?;

        json.as_array()
            .and_then(|values| values.first())
            .and_then(|value| serde_json::from_value::<SSHKeypair>(value.to_owned()).ok())
    }
}
