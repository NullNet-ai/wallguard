use crate::datastore::db_tables::DBTable;
use crate::datastore::{Datastore, Device};
use nullnet_libdatastore::{AdvanceFilterBuilder, BatchUpdateRequestBuilder, UpdateRequestBuilder};
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
        device_id: &str,
        is_online: bool,
    ) -> Result<(), Error> {
        let body = json!({
            "is_device_online": is_online
        })
        .to_string();

        let request = UpdateRequestBuilder::new()
            .id(device_id)
            .table(Device::table())
            .body(body)
            .performed_by_root(false)
            .build();

        let _ = self.inner.clone().update(request, token).await?;

        Ok(())
    }

    pub async fn update_all_devices_online_status(
        &self,
        token: &str,
        is_online: bool,
        performed_by_root: bool,
    ) -> Result<(), Error> {
        let update = json!({
            "is_device_online": is_online
        });

        let filter = AdvanceFilterBuilder::new()
            .field("is_device_online")
            .values("[true]")
            .r#type("criteria")
            .operator("equal")
            .entity(Device::table())
            .build();

        let request = BatchUpdateRequestBuilder::new()
            .advance_filter(filter)
            .performed_by_root(performed_by_root)
            .table(Device::table())
            .updates(update.to_string())
            .build();

        let _ = self.inner.clone().batch_update(request, token).await?;

        Ok(())
    }
}
