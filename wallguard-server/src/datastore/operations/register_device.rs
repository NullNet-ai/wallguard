use crate::datastore::{Datastore, builders::RegisterRequestBuilder};
use nullnet_libdatastore::{AccountType, RegisterResponse};
use nullnet_liberror::Error;

impl Datastore {
    pub async fn register_device(
        &self,
        token: &str,
        account_id: &str,
        account_secret: &str,
        organization_id: &str,
    ) -> Result<RegisterResponse, Error> {
        let request = RegisterRequestBuilder::new()
            .account_id(account_id)
            .account_secret(account_secret)
            .account_organization_status("Active")
            .account_status("Active")
            .is_new_user(true)
            .account_type(AccountType::Device)
            .add_account_organization_category("Device")
            .add_device_category("Device")
            .organization_id(organization_id)
            .set_is_request(true)
            .build();

        let response = self.inner.clone().register(request, token).await?;

        Ok(response)
    }
}
