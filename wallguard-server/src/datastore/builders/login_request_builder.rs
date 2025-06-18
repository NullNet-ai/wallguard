use nullnet_libdatastore::{LoginBody, LoginData, LoginParams, LoginRequest};

#[derive(Debug, Default)]
pub struct LoginRequestBuilder {
    account_id: Option<String>,
    account_secret: Option<String>,
    is_root: bool,
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

    pub fn set_root(mut self, is_root: bool) -> Self {
        self.is_root = is_root;
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
            params: Some(LoginParams {
                // Only need to do this because datastore defines this parameter as a String
                is_root: if self.is_root {
                    String::from("true")
                } else {
                    String::from("false")
                },
                t: String::new(),
            }),
        }
    }
}
