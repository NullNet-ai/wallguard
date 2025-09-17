use nullnet_libdatastore::CreateRequestBuilder;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use serde_json::json;

use crate::{
    datastore::{Datastore, Device},
    utilities,
};

impl Datastore {
    pub async fn create_device(&self, token: &str, device: &Device) -> Result<String, Error> {
        let mut json = json!(device);

        json.as_object_mut().unwrap().remove("id");

        let request = CreateRequestBuilder::new()
            .pluck(Device::pluck())
            .table(Device::table())
            .record(json.to_string())
            .build();

        let response = self.inner.clone().create(request, token).await?;

        let json_data = utilities::json::parse_string(&response.data)?;
        let value = utilities::json::first_element_from_array(&json_data)?;
        let retval = serde_json::from_value::<Device>(value).handle_err(location!())?;

        Ok(retval.id)
    }
}
