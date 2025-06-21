use nullnet_libdatastore::{AccountType, RegisterParams, RegisterRequest};

#[derive(Debug, Default)]
pub struct RegisterRequestBuilder {
    pub id: Option<String>,
    pub name: Option<String>,
    pub contact_id: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub parent_organization_id: Option<String>,
    pub code: Option<String>,
    pub categories: Vec<String>,
    pub account_status: Option<String>,
    pub account_type: Option<AccountType>,
    pub organization_name: Option<String>,
    pub organization_id: Option<String>,
    pub account_id: Option<String>,
    pub account_secret: Option<String>,
    pub is_new_user: bool,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_invited: bool,
    pub role_id: Option<String>,
    pub account_organization_status: Option<String>,
    pub account_organization_categories: Vec<String>,
    pub account_organization_id: Option<String>,
    pub contact_categories: Vec<String>,
    pub device_categories: Vec<String>,
    pub responsible_account_organization_id: Option<String>,
    pub is_request: bool,
}

impl RegisterRequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn contact_id(mut self, contact_id: impl Into<String>) -> Self {
        self.contact_id = Some(contact_id.into());
        self
    }

    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn parent_organization_id(mut self, id: impl Into<String>) -> Self {
        self.parent_organization_id = Some(id.into());
        self
    }

    pub fn code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn add_category(mut self, category: impl Into<String>) -> Self {
        self.categories.push(category.into());
        self
    }

    pub fn account_status(mut self, status: impl Into<String>) -> Self {
        self.account_status = Some(status.into());
        self
    }

    pub fn account_type(mut self, t: AccountType) -> Self {
        self.account_type = Some(t);
        self
    }

    pub fn organization_name(mut self, name: impl Into<String>) -> Self {
        self.organization_name = Some(name.into());
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

    pub fn first_name(mut self, name: impl Into<String>) -> Self {
        self.first_name = Some(name.into());
        self
    }

    pub fn last_name(mut self, name: impl Into<String>) -> Self {
        self.last_name = Some(name.into());
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

    pub fn account_organization_id(mut self, id: impl Into<String>) -> Self {
        self.account_organization_id = Some(id.into());
        self
    }

    pub fn add_contact_category(mut self, category: impl Into<String>) -> Self {
        self.contact_categories.push(category.into());
        self
    }

    pub fn add_device_category(mut self, category: impl Into<String>) -> Self {
        self.device_categories.push(category.into());
        self
    }

    pub fn responsible_account_organization_id(mut self, id: impl Into<String>) -> Self {
        self.responsible_account_organization_id = Some(id.into());
        self
    }

    pub fn set_is_request(mut self, is_request: bool) -> Self {
        self.is_request = is_request;
        self
    }

    pub fn build(self) -> RegisterRequest {
        let params = RegisterParams {
            id: self.id.unwrap_or_default(),
            name: self.name.unwrap_or_default(),
            contact_id: self.contact_id.unwrap_or_default(),
            email: self.email.unwrap_or_default(),
            password: self.password.unwrap_or_default(),
            parent_organization_id: self.parent_organization_id.unwrap_or_default(),
            code: self.code.unwrap_or_default(),
            categories: self.categories,
            account_status: self.account_status.unwrap_or_default(),
            account_type: self.account_type.unwrap_or(AccountType::Device).into(),
            organization_name: self.organization_name.unwrap_or_default(),
            organization_id: self.organization_id.unwrap_or_default(),
            account_id: self.account_id.unwrap_or_default(),
            account_secret: self.account_secret.unwrap_or_default(),
            is_new_user: self.is_new_user,
            first_name: self.first_name.unwrap_or_default(),
            last_name: self.last_name.unwrap_or_default(),
            is_invited: self.is_invited,
            role_id: self.role_id.unwrap_or_default(),
            account_organization_status: self.account_organization_status.unwrap_or_default(),
            account_organization_categories: self.account_organization_categories,
            account_organization_id: self.account_organization_id.unwrap_or_default(),
            contact_categories: self.contact_categories,
            device_categories: self.device_categories,
            responsible_account_organization_id: self
                .responsible_account_organization_id
                .unwrap_or_default(),
        };

        RegisterRequest {
            body: Some(params),
            is_request: self.is_request,
        }
    }
}
