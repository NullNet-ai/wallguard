use nullnet_libdatastore::ResponseData;
use std::collections::HashMap;

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
