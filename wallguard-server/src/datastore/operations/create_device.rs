use nullnet_liberror::Error;
use serde_json::json;

use crate::datastore::builders::CreateRequestBuilder;
use crate::datastore::{Datastore, Device};

impl Datastore {
    pub async fn create_device(
        &self,
        token: &str,
        device: &Device,
        org_id: Option<String>,
    ) -> Result<(), Error> {
        let mut json = json!(device);

        if let Some(org_id) = org_id {
            json["organization_id"] = json!(org_id);
        }

        let request = CreateRequestBuilder::new()
            .pluck(Device::pluck())
            .table(Device::table())
            .record(json.to_string())
            .build();

        let _ = self.inner.clone().create(request, token).await?;

        Ok(())
    }
}
