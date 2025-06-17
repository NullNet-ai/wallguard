use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Account {
    pub profile: Profile,
    pub organization: Organization,
    pub id: String,
    pub account_id: String,
    pub organization_id: String,
    pub account_status: String,
    pub contact: HashMap<String, serde_json::Value>,
    pub device: HashMap<String, serde_json::Value>,
    pub account_organization_id: Option<String>,
    pub role_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Profile {
    pub id: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: String,
    pub account_id: String,
    pub categories: Vec<String>,
    pub code: Option<String>,
    pub status: String,
    pub organization_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub code: String,
    pub categories: Vec<String>,
    pub status: String,
    pub organization_id: String,
    pub parent_organization_id: Option<String>,
}
