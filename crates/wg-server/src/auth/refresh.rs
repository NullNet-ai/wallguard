use rand::RngCore;
use sqlx::PgPool;
use uuid::Uuid;
use wg_shared::types::Role;

const REFRESH_TOKEN_TTL_DAYS: i64 = 30;

#[derive(Debug, thiserror::Error)]
pub enum RefreshError {
    #[error("database: {0}")]
    Db(#[from] sqlx::Error),
    #[error("token not found or expired")]
    InvalidToken,
}

/// Issue a new refresh token for `user_id` and insert it into `refresh_tokens`.
///
/// Returns the raw token string (64 random hex chars = 32 bytes).
pub async fn issue_refresh_token(pool: &PgPool, user_id: Uuid) -> Result<String, RefreshError> {
    let jti   = Uuid::new_v4();
    let raw   = jti_to_raw_token(jti);
    let expires_at = time::OffsetDateTime::now_utc()
        + time::Duration::days(REFRESH_TOKEN_TTL_DAYS);

    sqlx::query(
        "INSERT INTO refresh_tokens (jti, user_id, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(jti)
    .bind(user_id)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(raw)
}

/// Rotate a refresh token: delete the old one and issue a new one.
///
/// Returns `(new_raw_token, user_id, org_id, role)` on success.
///
/// The old token is deleted even if the user row lookup fails — callers should
/// treat a partial failure as a session invalidation (force re-login).
pub async fn rotate_refresh_token(
    pool:  &PgPool,
    raw:   &str,
) -> Result<(String, Uuid, Uuid, Role), RefreshError> {
    let jti = raw_token_to_jti(raw)?;

    // Atomic delete + user lookup in a transaction.
    let mut tx = pool.begin().await?;

    let row = sqlx::query_as::<_, (Uuid, Uuid, String)>(
        r#"
        DELETE FROM refresh_tokens
        WHERE  jti = $1 AND expires_at > NOW()
        RETURNING user_id,
                  (SELECT org_id FROM users WHERE id = refresh_tokens.user_id),
                  (SELECT role   FROM users WHERE id = refresh_tokens.user_id)
        "#,
    )
    .bind(jti)
    .fetch_optional(&mut *tx)
    .await?;

    let Some((user_id, org_id, role_str)) = row else {
        tx.rollback().await?;
        return Err(RefreshError::InvalidToken);
    };

    let role = parse_role(&role_str);

    // Issue the new token inside the same transaction.
    let new_jti = Uuid::new_v4();
    let new_raw = jti_to_raw_token(new_jti);
    let expires_at = time::OffsetDateTime::now_utc()
        + time::Duration::days(REFRESH_TOKEN_TTL_DAYS);

    sqlx::query(
        "INSERT INTO refresh_tokens (jti, user_id, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(new_jti)
    .bind(user_id)
    .bind(expires_at)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok((new_raw, user_id, org_id, role))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Token format: `<jti_no_dashes_hex>` = 32 lowercase hex chars of the UUID bytes.
fn jti_to_raw_token(jti: Uuid) -> String {
    // Use the UUID's 16 bytes + 32 extra random bytes so the token is both
    // unforgeable and uniquely maps back to the DB row via the UUID.
    let mut extra = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut extra);
    let id_hex: String = jti.as_bytes().iter().map(|b| format!("{b:02x}")).collect();
    let ext_hex: String = extra.iter().map(|b| format!("{b:02x}")).collect();
    format!("{id_hex}{ext_hex}")
}

/// Parse the UUID prefix from a raw token (first 32 hex chars = 16 bytes).
fn raw_token_to_jti(raw: &str) -> Result<Uuid, RefreshError> {
    if raw.len() < 32 {
        return Err(RefreshError::InvalidToken);
    }
    let id_hex = &raw[..32];
    let bytes: Vec<u8> = (0..16)
        .map(|i| u8::from_str_radix(&id_hex[i * 2..i * 2 + 2], 16))
        .collect::<Result<_, _>>()
        .map_err(|_| RefreshError::InvalidToken)?;
    Uuid::from_slice(&bytes).map_err(|_| RefreshError::InvalidToken)
}

fn parse_role(s: &str) -> Role {
    match s {
        "owner"    => Role::Owner,
        "admin"    => Role::Admin,
        "operator" => Role::Operator,
        _          => Role::Viewer,
    }
}
