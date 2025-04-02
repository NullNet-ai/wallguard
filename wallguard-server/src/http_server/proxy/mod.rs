mod auth;
mod http;
mod not_found;
mod websocket;

use crate::app_context::AppContext;
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Payload},
};
use not_found::NOT_FOUND_HTML;
use nullnet_libtunnel::Profile;

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

pub async fn proxy(
    request: HttpRequest,
    context: Data<AppContext>,
    body: Payload,
) -> actix_web::Result<HttpResponse> {
    log::info!("Proxy request: {request:?}");

    let Some(session) = extract_session_from_request(&request) else {
        return Ok(HttpResponse::NotFound().body(NOT_FOUND_HTML));
    };

    let Some(profile) = context
        .tunnel
        .lock()
        .await
        .get_profile_if_online_by_public_session_id(&session)
        .await
        .cloned()
    else {
        return Ok(HttpResponse::NotFound().body(NOT_FOUND_HTML));
    };

    let target = profile.get_visitor_addr();
    let vtoken = profile.get_visitor_token();

    if request
        .headers()
        .get(actix_web::http::header::SEC_WEBSOCKET_KEY)
        .is_some()
    {
        websocket::proxy_request(request, body, target, vtoken).await
    } else {
        http::proxy_request(request, body, target, vtoken, false).await
    }
}
