use crate::datastore::{Datastore, Device, builders::RegisterDeviceRequestBuilder};
use nullnet_libdatastore::Response;
use nullnet_liberror::Error;

impl Datastore {
    pub async fn register_device(
        &self,
        token: &str,
        account_id: &str,
        account_secret: &str,
        device: &Device,
    ) -> Result<Response, Error> {
        let request = RegisterDeviceRequestBuilder::new()
            .account_id(account_id)
            .account_secret(account_secret)
            .account_organization_status("Active")
            .is_new_user(true)
            .add_account_organization_category("Device")
            .add_device_category("Device")
            .organization_id(&device.organization)
            .device_id(&device.id)
            .build();

        let response = self.inner.clone().register_device(request, token).await?;

        Ok(response)
    }
}
