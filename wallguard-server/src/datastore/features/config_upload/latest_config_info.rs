use nullnet_libdatastore::ResponseData;
use nullnet_liberror::{location, Error, ErrorHandler, Location};

pub struct LatestConfigInfo {
    pub id: String,
    pub digest: String,
    pub version: i64,
}

impl LatestConfigInfo {
    pub fn from_response_data(data: &ResponseData) -> Result<Self, Error> {
        let json = serde_json::from_str::<serde_json::Value>(&data.data).handle_err(location!())?;
        Self::from_json(&json)
    }

    fn from_json(value: &serde_json::Value) -> Result<Self, Error> {
        let object = value
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|e| e.as_object())
            .ok_or("Unexpected value")
            .handle_err(location!())?;

        let digest = object
            .get("digest")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
            .ok_or("Unexpected value")
            .handle_err(location!())?;

        let id = object
            .get("id")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
            .ok_or("Unexpected value")
            .handle_err(location!())?;

        let version = object
            .get("config_version")
            .and_then(serde_json::Value::as_i64)
            .ok_or("Unexpected value")
            .handle_err(location!())?;

        Ok(Self {
            id,
            digest,
            version,
        })
    }
}
