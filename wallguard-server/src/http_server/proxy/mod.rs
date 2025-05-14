pub mod auth;
mod http;
mod not_found;
mod websocket;

use crate::{app_context::AppContext, http_server::common::extract_session_from_request};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Payload},
};
use not_found::NOT_FOUND_HTML;
use nullnet_libtunnel::Profile;

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
        let is_https = profile.ui_proto().to_lowercase() == "https";
        http::proxy_request(request, body, target, vtoken, is_https).await
    }
}
