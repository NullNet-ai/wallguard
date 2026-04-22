use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::RngCore;
use sqlx::PgPool;
use uuid::Uuid;
use wg_shared::types::Role;

#[derive(Debug, thiserror::Error)]
pub enum ApiKeyError {
    #[error("database: {0}")]
    Db(#[from] sqlx::Error),
    #[error("hash: {0}")]
    Hash(argon2::password_hash::Error),
}

impl From<argon2::password_hash::Error> for ApiKeyError {
    fn from(e: argon2::password_hash::Error) -> Self {
        Self::Hash(e)
    }
}

/// Create a new API key for `user_id` in `org_id`.
///
/// Returns `(key_id, raw_key)`.  The raw key is shown to the user exactly
/// once and never stored in plaintext — only its argon2id hash is persisted.
///
/// Raw key format: `wg_<id_hex32>_<secret_hex64>` (total ≈ 99 chars).
pub async fn create_api_key(
    pool:        &PgPool,
    org_id:      Uuid,
    user_id:     Uuid,
    description: Option<&str>,
    role:        Role,
) -> Result<(Uuid, String), ApiKeyError> {
    let id = Uuid::new_v4();

    let mut secret = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut secret);

    let raw = format_raw_key(id, &secret);
    let key_hash = hash_raw_key(&raw)?;
    let role_str = role_to_str(role);

    sqlx::query(
        r#"
        INSERT INTO api_keys (id, org_id, user_id, key_hash, description, role)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(id)
    .bind(org_id)
    .bind(user_id)
    .bind(&key_hash)
    .bind(description)
    .bind(role_str)
    .execute(pool)
    .await?;

    Ok((id, raw))
}

/// Validate a raw API key.
///
/// Parses the embedded key ID, loads the matching row, and verifies the hash.
/// Returns `(user_id, org_id, role)` on success, `None` if invalid or not found.
pub async fn validate_api_key(
    pool: &PgPool,
    raw:  &str,
) -> Result<Option<(Uuid, Uuid, Role)>, ApiKeyError> {
    let Some(key_id) = parse_key_id(raw) else {
        return Ok(None);
    };

    let row = sqlx::query_as::<_, (Uuid, Uuid, String, String)>(
        r#"
        SELECT user_id, org_id, key_hash, role
        FROM   api_keys
        WHERE  id = $1
          AND  (expires_at IS NULL OR expires_at > NOW())
        "#,
    )
    .bind(key_id)
    .fetch_optional(pool)
    .await?;

    let Some((user_id, org_id, key_hash, role_str)) = row else {
        return Ok(None);
    };

    let parsed = PasswordHash::new(&key_hash).map_err(ApiKeyError::Hash)?;
    if Argon2::default()
        .verify_password(raw.as_bytes(), &parsed)
        .is_err()
    {
        return Ok(None);
    }

    // Update last_used_at asynchronously; ignore errors (non-critical).
    let _ = sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
        .bind(key_id)
        .execute(pool)
        .await;

    Ok(Some((user_id, org_id, parse_role(&role_str))))
}

/// Delete (revoke) an API key by its ID.
pub async fn delete_api_key(pool: &PgPool, key_id: Uuid) -> Result<(), ApiKeyError> {
    sqlx::query("DELETE FROM api_keys WHERE id = $1")
        .bind(key_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn format_raw_key(id: Uuid, secret: &[u8; 32]) -> String {
    let id_hex: String     = id.as_bytes().iter().map(|b| format!("{b:02x}")).collect();
    let secret_hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();
    format!("wg_{id_hex}_{secret_hex}")
}

fn parse_key_id(raw: &str) -> Option<Uuid> {
    let rest = raw.strip_prefix("wg_")?;
    let (id_hex, _) = rest.split_once('_')?;
    if id_hex.len() != 32 {
        return None;
    }
    let bytes: Vec<u8> = (0..16)
        .map(|i| u8::from_str_radix(&id_hex[i * 2..i * 2 + 2], 16))
        .collect::<Result<_, _>>()
        .ok()?;
    Uuid::from_slice(&bytes).ok()
}

fn hash_raw_key(raw: &str) -> Result<String, argon2::password_hash::Error> {
    let salt   = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    Ok(argon2.hash_password(raw.as_bytes(), &salt)?.to_string())
}

fn role_to_str(role: Role) -> &'static str {
    match role {
        Role::Owner    => "owner",
        Role::Admin    => "admin",
        Role::Operator => "operator",
        Role::Viewer   => "viewer",
    }
}

fn parse_role(s: &str) -> Role {
    match s {
        "owner"    => Role::Owner,
        "admin"    => Role::Admin,
        "operator" => Role::Operator,
        _          => Role::Viewer,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_and_parse_key_id() {
        let id  = Uuid::new_v4();
        let raw = format_raw_key(id, &[0xab; 32]);
        let parsed = parse_key_id(&raw).unwrap();
        assert_eq!(parsed, id);
    }

    #[test]
    fn invalid_prefix_returns_none() {
        assert!(parse_key_id("notakey").is_none());
        assert!(parse_key_id("wg_tooshort_abc").is_none());
    }
}
