use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use uuid::Uuid;
use wg_shared::types::Role;

use crate::{
    auth::api_key,
    AppState,
};

/// Identity attached to every authenticated request.
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub user_id: Uuid,
    pub org_id:  Uuid,
    pub role:    Role,
}

/// Axum middleware: extract a `Bearer` JWT or an API key from the
/// `Authorization` header and attach a [`RequestContext`] extension.
///
/// Returns `401` if no valid credential is present.
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(token) = bearer_token(request.headers()) else {
        return unauthorized("missing or malformed Authorization header");
    };

    // --- Try JWT first ---
    if let Ok(claims) = state.jwt.validate(token).await {
        let ctx = RequestContext {
            user_id: claims.sub,
            org_id:  claims.org,
            role:    claims.role,
        };
        request.extensions_mut().insert(ctx);
        return next.run(request).await;
    }

    // --- Fall back to API key ---
    match api_key::validate_api_key(&state.pool, token).await {
        Ok(Some((user_id, org_id, role))) => {
            let ctx = RequestContext { user_id, org_id, role };
            request.extensions_mut().insert(ctx);
            next.run(request).await
        }
        Ok(None) => unauthorized("invalid credentials"),
        Err(e) => {
            tracing::error!("api key validation error: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "internal error"}))).into_response()
        }
    }
}

fn bearer_token(headers: &axum::http::HeaderMap) -> Option<&str> {
    let val = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    val.strip_prefix("Bearer ")
}

fn unauthorized(msg: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": { "message": msg, "code": "UNAUTHORIZED" } })),
    )
        .into_response()
}
