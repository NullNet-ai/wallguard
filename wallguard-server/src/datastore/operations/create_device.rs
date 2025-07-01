use nullnet_liberror::Error;
use serde_json::json;

use crate::datastore::{Datastore, Device, builders::CreateRequestBuilder};

impl Datastore {
    pub async fn create_device(&self, token: &str, device: &Device) -> Result<(), Error> {
        let mut json = json!(device);

        json.as_object_mut().unwrap().remove("id");

        let request = CreateRequestBuilder::new()
            .pluck(Device::pluck())
            .table(Device::table())
            .record(json.to_string())
            .build();

        let _ = self.inner.clone().create(request, token).await?;

        Ok(())
    }
}
