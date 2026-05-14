use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

const EXPIRATION_MARGIN: usize = 60 * 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub account_id: String,
    pub organization_id: String,
    pub account_organization_id: String,
    pub account_status: String,
    #[serde(default)]
    pub role_id: Option<String>,
    #[serde(default)]
    pub role_name: Option<String>,
    #[serde(default)]
    pub role_level: Option<u32>,
    #[serde(default)]
    pub is_root_account: bool,
    #[serde(default)]
    pub profile: Option<Value>,
    #[serde(default)]
    pub organization: Option<Value>,
    #[serde(default)]
    pub contact: Option<Value>,
    #[serde(default)]
    pub device: Option<Value>,
}

impl Account {
    pub fn device_id(&self) -> Option<&str> {
        self.device.as_ref()?.get("id")?.as_str()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Token {
    pub account: Account,
    pub sessionID: String,
    #[serde(default)]
    pub role_name: Option<String>,
    #[serde(default)]
    pub sensitivity_level: Option<u32>,
    #[serde(default)]
    pub previously_logged_in: Option<String>,
    pub signed_in_account: Account,
    pub exp: usize,
    pub iat: usize,
    #[serde(skip)]
    pub jwt: String,
}

impl Token {
    pub fn from_jwt(jwt: &str) -> Result<Self, String> {
        let parts: Vec<&str> = jwt.split('.').collect();

        if parts.len() != 3 {
            return Err(String::from("Malformed JWT"));
        }

        let decoded_payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|e| e.to_string())?;

        let mut token: Token =
            serde_json::from_slice(&decoded_payload).map_err(|e| e.to_string())?;
        token.jwt = jwt.to_string();

        Ok(token)
    }

    #[must_use]
    pub fn is_expired(&self) -> bool {
        let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) else {
            return true;
        };
        self.exp <= (duration.as_secs() as usize).saturating_sub(EXPIRATION_MARGIN)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct User {
    pub role_id: String,
    #[serde(default)]
    pub is_root_user: bool,
    pub account_id: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Origin {
    pub user_agent: Option<String>,
    pub host: String,
    pub url: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(non_snake_case)]
pub struct Cookie {
    pub path: String,
    pub expires: String,
    pub originalMaxAge: i64,
    pub httpOnly: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionPermissionCache {
    pub error: Option<Value>,
    pub cache_key: String,
    pub cached: Option<SessionPermissionCacheData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionPermissionCacheData {
    pub data: Option<Vec<Value>>,
    pub account_organization_id: Option<String>,
    pub cache: Option<bool>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Session {
    pub user: User,
    pub session_id: String,
    pub origin: Option<Origin>,
    pub token: String,
    pub cookie: Cookie,
    pub field_permissions: Option<SessionPermissionCache>,
    pub role_permissions: Option<SessionPermissionCache>,
    pub record_permissions: Option<SessionPermissionCache>,
    pub valid_pass_keys: Option<SessionPermissionCache>,
    #[serde(default)]
    pub ip_address: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub browser_name: Option<String>,
    #[serde(default)]
    pub operating_system: Option<String>,
    #[serde(default)]
    pub device_name: Option<String>,
    pub account_organization_id: Option<String>,
}
