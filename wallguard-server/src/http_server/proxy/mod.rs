mod http;
mod websocket;

mod not_found;
mod ws_proxy;

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

    let _pfsense = "192.168.2.46:80";
    let _tty = "192.168.2.52:3030";

    if request
        .headers()
        .get(actix_web::http::header::SEC_WEBSOCKET_KEY)
        .is_some()
    {
        websocket::proxy_request(request, body, _tty.parse().unwrap()).await
    } else {
        http::proxy_request(request, body, _pfsense.parse().unwrap()).await
    }
}
