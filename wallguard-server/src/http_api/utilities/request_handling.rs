use crate::http_api::utilities::authorization;
use crate::http_api::utilities::error_json::ErrorJson;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::web::Payload;
use actix_ws::{MessageStream, Session as WSSession};

pub fn extract_session_token(req: &HttpRequest) -> Result<String, HttpResponse> {
    // Primary path: tunnel ID encoded as the first subdomain label.
    // Used in production where a wildcard DNS record points every
    // <tunnel_id>.host to the server (e.g. abc123.wallguard.example.com).
    if let Some(token) = authorization::extract_proxy_session_token(req) {
        return Ok(token);
    }

    // Fallback: ?tunnel_id=<id> query parameter.
    // Useful when connecting directly to an IP or localhost without wildcard
    // DNS (e.g. during development or from the HTML test page).
    req.query_string()
        .split('&')
        .find_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            if k == "tunnel_id" {
                Some(v.to_owned())
            } else {
                None
            }
        })
        .ok_or_else(|| {
            HttpResponse::Unauthorized().json(ErrorJson::from("Session token is missing"))
        })
}

pub fn upgrade_to_websocket(
    request: HttpRequest,
    body: Payload,
) -> Result<(HttpResponse, WSSession, MessageStream), HttpResponse> {
    actix_ws::handle(&request, body)
        .map_err(|err| HttpResponse::InternalServerError().json(ErrorJson::from(err.to_string())))
}
