use axum::http::{HeaderMap, Request, StatusCode};
use axum::{extract::State, middleware::Next, response::Response};
use nullnet_libtoken::Token;
use std::sync::Arc;

use crate::{
    app_context::AppContext,
    datastore::{RemoteAccessSession, RemoteAccessType},
};

pub async fn authentication_middleware(
    State(context): State<AppContext>,
    headers: HeaderMap,
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let session_token = extract_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;

    let datastore_token = get_datastore_token(&context)
        .await
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let session = fetch_session_details(&context, datastore_token, &session_token)
        .await
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    if matches!(session.r#type, RemoteAccessType::Mcp) {
        request.extensions_mut().insert(session);
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

fn extract_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|auth_header| {
            auth_header
                .strip_prefix("Bearer ")
                .map(|stripped| stripped.to_string())
        })
}

async fn get_datastore_token(context: &AppContext) -> Option<Arc<Token>> {
    context.sysdev_token_provider.get().await.ok()
}

async fn fetch_session_details(
    context: &AppContext,
    token: Arc<Token>,
    session_token: &str,
) -> Option<RemoteAccessSession> {
    context
        .datastore
        .obtain_session(&token.jwt, session_token)
        .await
        .unwrap_or(None)
}
