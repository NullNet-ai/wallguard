use nullnet_libdatastore::{LoginBody, LoginData, LoginRequest};

#[derive(Default)]
pub struct LoginRequestBuilder {
    account_id: Option<String>,
    account_secret: Option<String>,
}

impl LoginRequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn account_id(mut self, id: impl Into<String>) -> Self {
        self.account_id = Some(id.into());
        self
    }

    pub fn account_secret(mut self, secret: impl Into<String>) -> Self {
        self.account_secret = Some(secret.into());
        self
    }

    pub fn build(self) -> LoginRequest {
        LoginRequest {
            body: Some(LoginBody {
                data: Some(LoginData {
                    account_id: self.account_id.unwrap_or_default(),
                    account_secret: self.account_secret.unwrap_or_default(),
                }),
            }),
        }
    }
}
