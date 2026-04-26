pub mod auth;
pub mod devices;
pub mod failures;
pub mod tunnels;
pub mod users;

use crate::auth::get_token;

/// Unified error type for all API calls.
pub type ApiResult<T> = Result<T, String>;

/// Base URL prefix — empty string so all paths are relative, letting
/// the dev-proxy (Trunk / nginx) forward `/api/…` to the backend.
pub fn api_base() -> &'static str {
    ""
}

/// GET `path`, deserializing the response body as JSON.
/// Attaches a `Bearer` token if one is present in localStorage.
pub async fn get<T: serde::de::DeserializeOwned>(path: &str) -> ApiResult<T> {
    let url = format!("{}{}", api_base(), path);

    let builder = gloo_net::http::Request::get(&url);
    let builder = if let Some(token) = get_token() {
        builder.header("Authorization", &format!("Bearer {token}"))
    } else {
        builder
    };

    let response = builder
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    let status = response.status();
    if !(200..300).contains(&status) {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }

    response
        .json::<T>()
        .await
        .map_err(|e| format!("Deserialize error: {e}"))
}

/// POST `path` with a JSON `body`, deserializing the response body as JSON.
/// Attaches a `Bearer` token and `Content-Type: application/json`.
pub async fn post<B: serde::Serialize, T: serde::de::DeserializeOwned>(
    path: &str,
    body: &B,
) -> ApiResult<T> {
    let url = format!("{}{}", api_base(), path);
    let body_str = serde_json::to_string(body).map_err(|e| format!("Serialize error: {e}"))?;

    let builder = gloo_net::http::Request::post(&url)
        .header("Content-Type", "application/json");
    let builder = if let Some(token) = get_token() {
        builder.header("Authorization", &format!("Bearer {token}"))
    } else {
        builder
    };

    let response = builder
        .body(body_str)
        .map_err(|e| format!("Request build error: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    let status = response.status();
    if !(200..300).contains(&status) {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }

    response
        .json::<T>()
        .await
        .map_err(|e| format!("Deserialize error: {e}"))
}

/// DELETE `path`, returning `()` on success.
/// Attaches a `Bearer` token if present.
pub async fn delete(path: &str) -> ApiResult<()> {
    let url = format!("{}{}", api_base(), path);

    let builder = gloo_net::http::Request::delete(&url);
    let builder = if let Some(token) = get_token() {
        builder.header("Authorization", &format!("Bearer {token}"))
    } else {
        builder
    };

    let response = builder
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    let status = response.status();
    if !(200..300).contains(&status) {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }

    Ok(())
}
