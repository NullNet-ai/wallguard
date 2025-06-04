use crate::datastore::builders::UpdateRequestBuilder;
use crate::datastore::db_tables::DBTable;
use crate::datastore::{Datastore, Device};
use nullnet_liberror::Error;
use serde_json::json;

impl Datastore {
    pub async fn update_device(
        &self,
        token: &str,
        device_id: &str,
        device: &Device,
    ) -> Result<bool, Error> {
        let request = UpdateRequestBuilder::new()
            .id(device_id)
            .table(DBTable::Devices)
            .body(json!(device).to_string())
            .build();

        let data = self.inner.clone().update(request, token).await?;

        Ok(data.count == 1)
    }
}
