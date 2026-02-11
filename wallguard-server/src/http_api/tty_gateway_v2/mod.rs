use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::rt;
use actix_web::web::{Data, Payload};

use super::utilities::error_json::ErrorJson;
use super::utilities::request_handling;
use crate::app_context::AppContext;
use crate::datastore::TtySessionStatus;
pub use crate::http_api::tty_gateway_v2::session::Session;
use crate::http_api::tty_gateway_v2::websocket_relay::websocket_relay;

mod internal_relay;
pub mod manager;
mod session;
mod websocket_relay;

pub(super) async fn open_tty_session(
    request: HttpRequest,
    context: Data<AppContext>,
    body: Payload,
) -> impl Responder {
    let session_id = match request_handling::extract_session_token(&request) {
        Ok(session_id) => session_id.to_ascii_uppercase(),
        Err(response) => return response,
    };

    let token = match request_handling::fetch_token(&context).await {
        Ok(t) => t,
        Err(resp) => return resp,
    };

    let Ok(Some(session)) = context
        .datastore
        .obtain_tty_session(&token.jwt, &session_id, false)
        .await
    else {
        return HttpResponse::NotFound().json(ErrorJson::from("Failed to obtain tty session data"));
    };

    if !matches!(session.session_status, TtySessionStatus::Active) {
        return HttpResponse::BadRequest().json(ErrorJson::from("Session is not active"));
    }

    let Ok(Some(device)) = context
        .datastore
        .obtain_device_by_id(&token.jwt, &session.device_id, false)
        .await
    else {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Unable to retrieve device from datastore"));
    };

    if !device.authorized {
        return HttpResponse::BadRequest().json(ErrorJson::from("Device is unauthorized"));
    }

    let Some(session_intance) = context.tty_sessions_manager.get_by_id(&session_id).await else {
        return HttpResponse::NotFound().json(ErrorJson::from("Session not found"));
    };

    let (response, ws_session, stream) = match request_handling::upgrade_to_websocket(request, body)
    {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    rt::spawn(websocket_relay(stream, ws_session, session_intance));

    response
}
