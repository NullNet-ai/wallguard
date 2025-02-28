use crate::datastore::DatastoreWrapper;
use nullnet_libdatastore::{LoginBody, LoginData, LoginRequest};
use nullnet_liberror::Error;

impl DatastoreWrapper {
    pub async fn login(&self, account_id: String, account_secret: String) -> Result<String, Error> {
        let request = LoginRequest {
            body: Some(LoginBody {
                data: Some(LoginData {
                    account_id,
                    account_secret,
                }),
            }),
        };

        let response = self.inner.clone().login(request).await?;

        Ok(response.token)
    }
}
