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
    pub profile: Option<serde_json::Value>,
    #[serde(default)]
    pub organization: Option<serde_json::Value>,
    #[serde(default)]
    pub contact: Option<serde_json::Value>, 
    #[serde(default)]
    pub device: Option<serde_json::Value>,
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
    pub sessionID: Option<String>,
    #[serde(default)]
    pub role_name: Option<String>,
    #[serde(default)]
    pub sensitivity_level: Option<u32>,
    #[serde(default)]
    pub previously_logged_in: Option<String>,
    pub signed_in_account: Account,
    exp: usize,
    iat: usize,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token() {
        let jwt = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJhY2NvdW50Ijp7ImFjY291bnRfaWQiOiJxd2JxNDZqcWNsZXYiLCJhY2NvdW50X29yZ2FuaXphdGlvbl9pZCI6bnVsbCwiYWNjb3VudF9zdGF0dXMiOiJBY3RpdmUiLCJjb250YWN0Ijp7fSwiZGV2aWNlIjp7fSwiaWQiOiIwMUtSUE41RUpLUEhUSldUM1Y2WUM3NjhXQSIsIm9yZ2FuaXphdGlvbiI6eyJjYXRlZ29yaWVzIjpbIlBlcnNvbmFsIl0sImNvZGUiOiJPMDAwMDIxIiwiaWQiOiIwMUtSUE41RVA3QTdOWjJWQTU2SjBBOVNZSyIsIm5hbWUiOiJQZXJzb25hbCBPcmdhbml6YXRpb24iLCJvcmdhbml6YXRpb25faWQiOiIwMUtSUE41RVA3QTdOWjJWQTU2SjBBOVNZSyIsInBhcmVudF9vcmdhbml6YXRpb25faWQiOm51bGwsInN0YXR1cyI6IkFjdGl2ZSJ9LCJvcmdhbml6YXRpb25faWQiOiIwMUtSUE41RVA3QTdOWjJWQTU2SjBBOVNZSyIsInByb2ZpbGUiOnsiYWNjb3VudF9pZCI6IjAxS1JQTjVFSktQSFRKV1QzVjZZQzc2OFdBIiwiY2F0ZWdvcmllcyI6W10sImNvZGUiOm51bGwsImVtYWlsIjoicXdicTQ2anFjbGV2IiwiZmlyc3RfbmFtZSI6IiIsImlkIjoiMDFLUlBONUZCQ1cwRU5GWDNDNTBGOE01TVYiLCJsYXN0X25hbWUiOiIiLCJvcmdhbml6YXRpb25faWQiOiIwMUtSUE41RVA3QTdOWjJWQTU2SjBBOVNZSyIsInN0YXR1cyI6IkFjdGl2ZSJ9LCJyb2xlX2lkIjpudWxsLCJzZXNzaW9uSUQiOiIifSwiZXhwIjoxNzc4OTYzOTM4LCJpYXQiOjE3Nzg4Nzc1MzgsInJvbGVfbmFtZSI6IiIsInNlbnNpdGl2aXR5X2xldmVsIjoxMDAwLCJzZXNzaW9uSUQiOiIiLCJzaWduZWRfaW5fYWNjb3VudCI6eyJhY2NvdW50X2lkIjoicXdicTQ2anFjbGV2IiwiYWNjb3VudF9vcmdhbml6YXRpb25faWQiOm51bGwsImFjY291bnRfc3RhdHVzIjoiQWN0aXZlIiwiY29udGFjdCI6e30sImRldmljZSI6e30sImlkIjoiMDFLUlBONUVKS1BIVEpXVDNWNllDNzY4V0EiLCJvcmdhbml6YXRpb24iOnsiY2F0ZWdvcmllcyI6WyJQZXJzb25hbCJdLCJjb2RlIjoiTzAwMDAyMSIsImlkIjoiMDFLUlBONUVQN0E3TloyVkE1NkowQTlTWUsiLCJuYW1lIjoiUGVyc29uYWwgT3JnYW5pemF0aW9uIiwib3JnYW5pemF0aW9uX2lkIjoiMDFLUlBONUVQN0E3TloyVkE1NkowQTlTWUsiLCJwYXJlbnRfb3JnYW5pemF0aW9uX2lkIjpudWxsLCJzdGF0dXMiOiJBY3RpdmUifSwib3JnYW5pemF0aW9uX2lkIjoiMDFLUlBONUVQN0E3TloyVkE1NkowQTlTWUsiLCJwcm9maWxlIjp7ImFjY291bnRfaWQiOiIwMUtSUE41RUpLUEhUSldUM1Y2WUM3NjhXQSIsImNhdGVnb3JpZXMiOltdLCJjb2RlIjpudWxsLCJlbWFpbCI6InF3YnE0NmpxY2xldiIsImZpcnN0X25hbWUiOiIiLCJpZCI6IjAxS1JQTjVGQkNXMEVORlgzQzUwRjhNNU1WIiwibGFzdF9uYW1lIjoiIiwib3JnYW5pemF0aW9uX2lkIjoiMDFLUlBONUVQN0E3TloyVkE1NkowQTlTWUsiLCJzdGF0dXMiOiJBY3RpdmUifSwicm9sZV9pZCI6bnVsbCwic2Vzc2lvbklEIjoiIn19.uCwpxbfDo6-3v-2hkbgPisbEo0GzMaQUv9SxKXIhWfo";

        let result = Token::from_jwt(&jwt);

        assert!(result.is_ok());
    }
}