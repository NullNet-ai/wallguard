use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

/// Axum middleware: read or generate `X-Request-Id`, propagate to response.
///
/// If the incoming request already has an `X-Request-Id` header it is reused;
/// otherwise a fresh UUID v4 is generated.  The header is echoed back in the
/// response so that clients can correlate logs.
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let id: String = request
        .headers()
        .get(&X_REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Normalise the incoming header (or add it if absent) so downstream
    // handlers can read it from extensions or the header map.
    if let Ok(val) = HeaderValue::from_str(&id) {
        request.headers_mut().insert(X_REQUEST_ID.clone(), val);
    }

    let span = tracing::info_span!("request", request_id = %id);
    let _guard = span.enter();

    let mut response = next.run(request).await;

    if let Ok(val) = HeaderValue::from_str(&id) {
        response.headers_mut().insert(X_REQUEST_ID.clone(), val);
    }

    response
}
