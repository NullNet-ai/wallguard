use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

// Vite outputs to ui/dist; path is relative to this crate's
// CARGO_MANIFEST_DIR (crates/wg-server).
#[derive(RustEmbed)]
#[folder = "../../ui/dist"]
struct UiAssets;

pub async fn handler(uri: Uri) -> Response {
    let raw = uri.path().trim_start_matches('/');

    // Exact-match the requested path, then fall back to index.html (SPA routing).
    let (path, asset) = match UiAssets::get(raw).filter(|_| !raw.is_empty()) {
        Some(a) => (raw.to_owned(), a),
        None => match UiAssets::get("index.html") {
            Some(a) => ("index.html".to_owned(), a),
            None    => return StatusCode::NOT_FOUND.into_response(),
        },
    };

    let cache = if path == "index.html" {
        "no-cache"
    } else {
        "public, max-age=31536000, immutable"
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_for(&path))
        .header(header::CACHE_CONTROL, cache)
        .body(Body::from(asset.data))
        .unwrap()
}

fn mime_for(path: &str) -> &'static str {
    match path.rsplit_once('.').map(|(_, ext)| ext) {
        Some("html") => "text/html; charset=utf-8",
        Some("js")   => "application/javascript; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("css")  => "text/css; charset=utf-8",
        Some("svg")  => "image/svg+xml",
        Some("png")  => "image/png",
        Some("ico")  => "image/x-icon",
        Some("json") => "application/json",
        _            => "application/octet-stream",
    }
}
