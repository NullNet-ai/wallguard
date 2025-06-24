use crate::datastore::builders::{
    AdvanceFilterBuilder, CreateRequestBuilder, DeleteRequestBuilder, GetByFilterRequestBuilder,
};
use crate::datastore::{Datastore, DeviceCredentials};
use crate::utilities::json;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use serde_json::json;

impl Datastore {
    pub async fn create_device_credentials(
        &self,
        token: &str,
        credentials: &DeviceCredentials,
    ) -> Result<(), Error> {
        let mut json = json!(credentials);

        // ID will be automatically generated
        json.as_object_mut().unwrap().remove("id");

        let request = CreateRequestBuilder::new()
            .pluck(DeviceCredentials::pluck())
            .table(DeviceCredentials::table())
            .record(json.to_string())
            .build();

        let _ = self.inner.clone().create(request, token).await?;

        Ok(())
    }

    pub async fn obtain_device_credentials(
        &self,
        token: &str,
        device_uuid: &str,
    ) -> Result<Option<DeviceCredentials>, Error> {
        let filter = AdvanceFilterBuilder::new()
            .field("device_uuid")
            .values(format!("[\"{device_uuid}\"]"))
            .r#type("criteria")
            .operator("equal")
            .entity(DeviceCredentials::table())
            .build();

        let request = GetByFilterRequestBuilder::new()
            .table(DeviceCredentials::table())
            .plucks(DeviceCredentials::pluck())
            .limit(1)
            .advance_filter(filter)
            .build();

        let response = self.inner.clone().get_by_filter(request, token).await?;

        if response.count == 0 {
            return Ok(None);
        }

        let json_data = json::parse_string(&response.data)?;
        let data = json::first_element_from_array(&json_data)?;

        let device = serde_json::from_value::<DeviceCredentials>(data).handle_err(location!())?;
        Ok(Some(device))
    }

    pub async fn delete_device_credentials(&self, token: &str, id: &str) -> Result<(), Error> {
        let request = DeleteRequestBuilder::new()
            .id(id)
            .table(DeviceCredentials::table())
            .permanent(true)
            .build();

        let _ = self.inner.clone().delete(request, token).await?;
        Ok(())
    }
}
