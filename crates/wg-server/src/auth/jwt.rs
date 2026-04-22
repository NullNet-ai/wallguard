use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use wg_shared::types::Role;

const SECRET_KEY_NAME:    &str = "jwt_signing_key";
const ACCESS_TOKEN_TTL_S: u64  = 3600;  // 1 hour

#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("database: {0}")]
    Db(#[from] sqlx::Error),
    #[error("token: {0}")]
    Token(#[from] jsonwebtoken::errors::Error),
    #[error("token revoked")]
    Revoked,
}

/// Claims embedded in every access token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject: user UUID.
    pub sub:  Uuid,
    /// Organisation UUID.
    pub org:  Uuid,
    pub role: Role,
    /// JWT ID — stored in `revoked_tokens` on logout/rotation.
    pub jti:  Uuid,
    /// Expiry (Unix seconds).
    pub exp:  u64,
    /// Issued-at (Unix seconds).
    pub iat:  u64,
}

/// Stateful JWT service — holds the signing key and checks revocation via DB.
#[derive(Clone)]
pub struct JwtService {
    enc:  EncodingKey,
    dec:  DecodingKey,
    pool: PgPool,
}

impl JwtService {
    /// Initialise the service.  Loads the signing key from `server_secrets`;
    /// if no row exists a fresh 64-byte random key is generated and persisted.
    pub async fn new(pool: PgPool) -> Result<Self, JwtError> {
        let key_bytes = load_or_create_signing_key(&pool).await?;
        Ok(Self {
            enc:  EncodingKey::from_secret(&key_bytes),
            dec:  DecodingKey::from_secret(&key_bytes),
            pool,
        })
    }

    /// Issue a new access token.  The `jti` is a fresh random UUID each call.
    pub fn issue(&self, user_id: Uuid, org_id: Uuid, role: Role) -> Result<String, JwtError> {
        issue_with_key(user_id, org_id, role, &self.enc)
    }

    /// Validate a token: verify signature, expiry, and revocation status.
    pub async fn validate(&self, token: &str) -> Result<Claims, JwtError> {
        let claims = validate_signature(token, &self.dec)?;

        let revoked: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM revoked_tokens WHERE jti = $1)",
        )
        .bind(claims.jti)
        .fetch_one(&self.pool)
        .await?;

        if revoked {
            return Err(JwtError::Revoked);
        }

        Ok(claims)
    }

    /// Revoke a token by inserting its JTI into `revoked_tokens`.
    pub async fn revoke(&self, claims: &Claims) -> Result<(), JwtError> {
        let expires_at = time::OffsetDateTime::from_unix_timestamp(claims.exp as i64)
            .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
        sqlx::query(
            "INSERT INTO revoked_tokens (jti, expires_at) VALUES ($1, $2)
             ON CONFLICT DO NOTHING",
        )
        .bind(claims.jti)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Pure helpers — also used directly in unit tests without a real pool.
// ---------------------------------------------------------------------------

pub fn issue_with_key(
    user_id: Uuid,
    org_id:  Uuid,
    role:    Role,
    enc:     &EncodingKey,
) -> Result<String, JwtError> {
    let now = unix_now();
    let claims = Claims {
        sub:  user_id,
        org:  org_id,
        role,
        jti:  Uuid::new_v4(),
        exp:  now + ACCESS_TOKEN_TTL_S,
        iat:  now,
    };
    Ok(encode(&Header::new(Algorithm::HS256), &claims, enc)?)
}

pub fn validate_signature(token: &str, dec: &DecodingKey) -> Result<Claims, JwtError> {
    let mut val = Validation::new(Algorithm::HS256);
    val.validate_exp = true;
    Ok(decode::<Claims>(token, dec, &val)?.claims)
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before epoch")
        .as_secs()
}

async fn load_or_create_signing_key(pool: &PgPool) -> Result<Vec<u8>, JwtError> {
    let row: Option<Vec<u8>> = sqlx::query_scalar(
        "SELECT value FROM server_secrets WHERE key = $1",
    )
    .bind(SECRET_KEY_NAME)
    .fetch_optional(pool)
    .await?;

    if let Some(bytes) = row {
        return Ok(bytes);
    }

    let mut key = vec![0u8; 64];
    rand::thread_rng().fill_bytes(&mut key);

    sqlx::query(
        "INSERT INTO server_secrets (key, value) VALUES ($1, $2)
         ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
    )
    .bind(SECRET_KEY_NAME)
    .bind(&key)
    .execute(pool)
    .await?;

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY: &[u8] = b"test_key_at_least_32_bytes_long!!";

    #[test]
    fn issue_and_decode_round_trip() {
        let enc     = EncodingKey::from_secret(TEST_KEY);
        let dec     = DecodingKey::from_secret(TEST_KEY);
        let user_id = Uuid::new_v4();
        let org_id  = Uuid::new_v4();
        let token   = issue_with_key(user_id, org_id, Role::Admin, &enc).unwrap();
        let claims  = validate_signature(&token, &dec).unwrap();

        assert_eq!(claims.sub,  user_id);
        assert_eq!(claims.org,  org_id);
        assert_eq!(claims.role, Role::Admin);
    }

    #[test]
    fn wrong_key_rejected() {
        let enc     = EncodingKey::from_secret(TEST_KEY);
        let token   = issue_with_key(Uuid::new_v4(), Uuid::new_v4(), Role::Viewer, &enc).unwrap();
        let bad_dec = DecodingKey::from_secret(b"wrong_key_minimum_32_bytes_long!!");
        assert!(validate_signature(&token, &bad_dec).is_err());
    }

    #[test]
    fn expired_token_rejected() {
        let enc = EncodingKey::from_secret(TEST_KEY);
        let dec = DecodingKey::from_secret(TEST_KEY);

        let claims = Claims {
            sub:  Uuid::new_v4(),
            org:  Uuid::new_v4(),
            role: Role::Operator,
            jti:  Uuid::new_v4(),
            exp:  1,  // Unix epoch + 1s — always in the past
            iat:  1,
        };
        let token = encode(&Header::new(Algorithm::HS256), &claims, &enc).unwrap();
        let err   = validate_signature(&token, &dec).unwrap_err();

        assert!(matches!(
            err,
            JwtError::Token(ref e)
            if matches!(e.kind(), jsonwebtoken::errors::ErrorKind::ExpiredSignature)
        ));
    }

    #[test]
    fn each_token_has_unique_jti() {
        let enc = EncodingKey::from_secret(TEST_KEY);
        let dec = DecodingKey::from_secret(TEST_KEY);
        let uid = Uuid::new_v4();
        let oid = Uuid::new_v4();
        let t1  = issue_with_key(uid, oid, Role::Viewer, &enc).unwrap();
        let t2  = issue_with_key(uid, oid, Role::Viewer, &enc).unwrap();
        let c1  = validate_signature(&t1, &dec).unwrap();
        let c2  = validate_signature(&t2, &dec).unwrap();
        assert_ne!(c1.jti, c2.jti);
    }
}
