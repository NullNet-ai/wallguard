use crate::datastore::builders::{
    AdvanceFilterBuilder, BatchUpdateRequestBuilder, UpdateRequestBuilder,
};
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

    pub async fn update_device_online_status(
        &self,
        token: &str,
        device_uuid: &str,
        is_online: bool,
    ) -> Result<(), Error> {
        let updates = json!({
            "is_device_online": is_online
        })
        .to_string();

        let filter = AdvanceFilterBuilder::new()
            .field("device_uuid")
            .values(format!("[\"{device_uuid}\"]"))
            .r#type("criteria")
            .operator("equal")
            .entity(Device::table())
            .build();

        let request = BatchUpdateRequestBuilder::new()
            .table(Device::table())
            .updates(updates)
            .advance_filter(filter)
            .build();

        let _ = self.inner.clone().batch_update(request, token).await;

        Ok(())
    }
}
