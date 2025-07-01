use nullnet_libdatastore::{RegisterDeviceParams, RegisterDeviceRequest};

#[derive(Debug, Default)]
pub struct RegisterDeviceRequestBuilder {
    pub categories: Vec<String>,
    pub organization_id: Option<String>,
    pub account_id: Option<String>,
    pub account_secret: Option<String>,
    pub is_new_user: bool,
    pub is_invited: bool,
    pub role_id: Option<String>,
    pub account_organization_status: Option<String>,
    pub account_organization_categories: Vec<String>,
    pub device_categories: Vec<String>,
    pub device_id: Option<String>,
}

impl RegisterDeviceRequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn device_id(mut self, device_id: impl Into<String>) -> Self {
        self.device_id = Some(device_id.into());
        self
    }

    pub fn add_category(mut self, category: impl Into<String>) -> Self {
        self.categories.push(category.into());
        self
    }

    pub fn organization_id(mut self, id: impl Into<String>) -> Self {
        self.organization_id = Some(id.into());
        self
    }

    pub fn account_id(mut self, id: impl Into<String>) -> Self {
        self.account_id = Some(id.into());
        self
    }

    pub fn account_secret(mut self, secret: impl Into<String>) -> Self {
        self.account_secret = Some(secret.into());
        self
    }

    pub fn is_new_user(mut self, flag: bool) -> Self {
        self.is_new_user = flag;
        self
    }

    pub fn is_invited(mut self, flag: bool) -> Self {
        self.is_invited = flag;
        self
    }

    pub fn role_id(mut self, id: impl Into<String>) -> Self {
        self.role_id = Some(id.into());
        self
    }

    pub fn account_organization_status(mut self, status: impl Into<String>) -> Self {
        self.account_organization_status = Some(status.into());
        self
    }

    pub fn add_account_organization_category(mut self, category: impl Into<String>) -> Self {
        self.account_organization_categories.push(category.into());
        self
    }

    pub fn add_device_category(mut self, category: impl Into<String>) -> Self {
        self.device_categories.push(category.into());
        self
    }

    pub fn build(self) -> RegisterDeviceRequest {
        RegisterDeviceRequest {
            device: Some(RegisterDeviceParams {
                organization_id: self.organization_id.unwrap_or_default(),
                account_id: self.account_id.unwrap_or_default(),
                account_secret: self.account_secret.unwrap_or_default(),
                is_new_user: self.is_new_user,
                is_invited: self.is_invited,
                role_id: self.role_id.unwrap_or_default(),
                account_organization_status: self.account_organization_status.unwrap_or_default(),
                account_organization_categories: self.account_organization_categories,
                device_categories: self.device_categories,
                device_id: self.device_id.unwrap_or_default(),
            }),
        }
    }
}
