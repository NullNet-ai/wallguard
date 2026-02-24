use crate::http_api::utilities::authorization;
use crate::http_api::utilities::error_json::ErrorJson;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::web::Payload;
use actix_ws::{MessageStream, Session as WSSession};

pub fn extract_session_token(req: &HttpRequest) -> Result<String, HttpResponse> {
    authorization::extract_proxy_session_token(req).ok_or_else(|| {
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
