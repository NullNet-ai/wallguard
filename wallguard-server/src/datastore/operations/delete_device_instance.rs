use nullnet_libdatastore::{AdvanceFilterBuilder, BatchDeleteRequestBuilder, DeleteRequestBuilder};
use nullnet_liberror::Error;

use crate::datastore::{Datastore, DeviceInstance};

impl Datastore {
    pub async fn delete_device_instance(
        &self,
        token: &str,
        instance_id: &str,
    ) -> Result<(), Error> {
        let request = DeleteRequestBuilder::new()
            .id(instance_id)
            .table(DeviceInstance::table())
            .build();

        let _ = self.inner.clone().delete(request, token).await?;

        Ok(())
    }

    pub async fn delete_all_device_instances(
        &self,
        token: &str,
        perfomed_by_root: bool,
    ) -> Result<(), Error> {
        let filter = AdvanceFilterBuilder::new()
            .field("status")
            .values("[\"Active\"]")
            .r#type("criteria")
            .operator("equal")
            .entity(DeviceInstance::table())
            .build();

        let request = BatchDeleteRequestBuilder::new()
            .table(DeviceInstance::table())
            .advance_filter(filter)
            .performed_by_root(perfomed_by_root)
            .build();

        let _ = self.inner.clone().batch_delete(request, token).await?;

        Ok(())
    }
}
