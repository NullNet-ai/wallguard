use base64::engine::Engine;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AuthToken {
    token: String,
    exp: u64,
    leeway: u64,
}

impl AuthToken {
    pub fn new(token: String, leeway: u64) -> Result<Self, String> {
        let payload = decode_payload(token.as_str())?;

        let json: serde_json::Value = serde_json::from_str(&payload).map_err(|e| e.to_string())?;

        let exp = json["exp"]
            .as_u64()
            .ok_or_else(|| "Missing or invalid 'exp' field".to_string())?;

        Ok(Self { token, exp, leeway })
    }

    pub fn is_expired(&self) -> bool {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        self.exp <= (current_time + self.leeway)
    }

    pub fn as_str(&self) -> &str {
        self.token.as_str()
    }
}

fn decode_payload(token: &str) -> Result<String, String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid JWT format".to_string());
    }

    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|e| format!("Failed to decode JWT payload: {e}"))?;

    String::from_utf8(decoded)
        .map_err(|e| format!("Failed to construct string from decoded JWT payload: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_mock_token(exp: u64) -> String {
        let payload = json!({ "exp": exp });
        let payload_base64 =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload.to_string());
        format!("header.{}.signature", payload_base64)
    }

    #[test]
    fn test_new_auth_token() {
        let exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600;

        let token = create_mock_token(exp);
        let auth_token = AuthToken::new(token.clone(), 5);

        assert!(auth_token.is_ok());
        let auth_token = auth_token.unwrap();
        assert_eq!(auth_token.exp, exp);
        assert_eq!(auth_token.as_str(), token);
    }

    #[test]
    fn test_is_expired() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let future_exp = now + 3600;

        let valid_token = create_mock_token(future_exp);
        let valid_auth_token = AuthToken::new(valid_token, 5).unwrap();
        assert!(!valid_auth_token.is_expired());

        let past_exp = now - 10;
        let expired_token = create_mock_token(past_exp);
        let expired_auth_token = AuthToken::new(expired_token, 5).unwrap();
        assert!(expired_auth_token.is_expired());
    }

    #[test]
    fn test_invalid_token_format() {
        let invalid_token = "invalid.token.format".to_string();
        let auth_token = AuthToken::new(invalid_token, 5);

        assert!(auth_token.is_err());
    }

    #[test]
    fn test_missing_fields_in_payload() {
        let payload = json!({});
        let payload_base64 =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload.to_string());
        let token = format!("header.{}.signature", payload_base64);

        let auth_token = AuthToken::new(token, 5);
        assert!(auth_token.is_err());
    }

    #[test]
    fn test_invalid_base64_encoding() {
        let token = "header.invalidpayload.signature".to_string();
        let auth_token = AuthToken::new(token, 5);

        assert!(auth_token.is_err());
    }
}
