mod http_proxy;
mod not_found;

use actix_web::{web::Payload, HttpRequest, HttpResponse};
use not_found::NOT_FOUND_HTML;

// @TODO:
// full url replace the pre-defined domain and be left with the session
fn extract_session_from_domain(domain: &str) -> Option<&str> {
    domain.split_once('.').map(|(session, _)| session)
}

fn extract_session_from_request(req: &HttpRequest) -> Option<String> {
    req.full_url()
        .domain()
        .and_then(extract_session_from_domain)
        .map(|v| v.to_owned())
}

pub async fn proxy(request: HttpRequest, body: Payload) -> actix_web::Result<HttpResponse> {
    let Some(session) = extract_session_from_request(&request) else {
        return Ok(HttpResponse::NotFound().body(NOT_FOUND_HTML));
    };

    // @TODO: validate session and get the sockaddr from it
    http_proxy::proxy_http_request(request, body, "192.168.2.52:5000".parse().unwrap()).await
}
