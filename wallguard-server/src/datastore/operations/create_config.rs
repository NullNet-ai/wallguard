use crate::datastore::builders::CreateRequestBuilder;
use crate::datastore::{Datastore, DeviceConfiguration};
use crate::utilities;
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn create_config(
        &self,
        token: &str,
        config: &DeviceConfiguration,
    ) -> Result<String, Error> {
        let mut json = serde_json::to_value(config).handle_err(location!())?;

        json.as_object_mut().unwrap().remove("id");

        let request = CreateRequestBuilder::new()
            .pluck(DeviceConfiguration::pluck())
            .table(DeviceConfiguration::table())
            .record(json.to_string())
            .build();

        let response = self.inner.clone().create(request, token).await?;

        if response.count == 1 {
            let json_data = utilities::json::parse_string(&response.data)?;
            let data = utilities::json::first_element_from_array(&json_data)?;

            let id = data["id"]
                .as_str()
                .ok_or("Missing or invalid 'id' field")
                .handle_err(location!())?
                .to_string();

            Ok(id)
        } else {
            Err("Failed to create device configuration").handle_err(location!())
        }
    }
}
