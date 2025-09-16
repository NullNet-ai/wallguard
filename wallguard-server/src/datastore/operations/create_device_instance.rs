use nullnet_libdatastore::CreateRequestBuilder;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use serde_json::json;

use crate::{
    datastore::{Datastore, DeviceInstance},
    utilities,
};

impl Datastore {
    pub async fn create_device_instance(
        &self,
        token: &str,
        instance: &DeviceInstance,
    ) -> Result<String, Error> {
        let mut json = json!(instance);
        json.as_object_mut().unwrap().remove("id");

        let request = CreateRequestBuilder::new()
            .pluck(DeviceInstance::pluck())
            .table(DeviceInstance::table())
            .record(json.to_string())
            .build();

        let response = self.inner.clone().create(request, token).await?;

        let json_data = utilities::json::parse_string(&response.data)?;
        let value = utilities::json::first_element_from_array(&json_data)?;
        let retval = serde_json::from_value::<DeviceInstance>(value).handle_err(location!())?;

        Ok(retval.id.clone())
    }
}
