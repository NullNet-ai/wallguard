use tonic::Request;

use crate::datastore::DatastoreWrapper;
use nullnet_libdatastore::{Error as DSError, LoginBody, LoginData, LoginRequest};

impl DatastoreWrapper {
    pub async fn login(
        &self,
        account_id: String,
        account_secret: String,
    ) -> Result<String, DSError> {
        let request = Request::new(LoginRequest {
            body: Some(LoginBody {
                data: Some(LoginData {
                    account_id,
                    account_secret,
                }),
            }),
        });

        let response = self.inner.login(request).await?;

        Ok(response.token)
    }
}
