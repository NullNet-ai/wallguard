use nullnet_liberror::Error;

use crate::datastore::{Datastore, DeviceInstance, builders::DeleteRequestBuilder};

impl Datastore {
    pub async fn delete_device_instance(
        &self,
        token: &str,
        instance: &DeviceInstance,
    ) -> Result<(), Error> {
        let request = DeleteRequestBuilder::new()
            .id(instance.id.clone())
            .table(DeviceInstance::table())
            .build();

        let _ = self.inner.clone().delete(request, token).await?;

        Ok(())
    }
}
