use nullnet_libdatastore::ResponseData;
use nullnet_liberror::{location, Error, ErrorHandler, Location};

pub fn parse_configuraion_id(response: &ResponseData) -> Result<String, Error> {
    let json: serde_json::Value = serde_json::from_str(&response.data).handle_err(location!())?;

    json.as_array()
        .and_then(|arr| arr.first())
        .and_then(|obj| obj.as_object())
        .and_then(|map| map.get("id"))
        .and_then(|v| v.as_str())
        .map(std::string::ToString::to_string)
        .ok_or(format!("Failed to parse"))
        .handle_err(location!())
}
