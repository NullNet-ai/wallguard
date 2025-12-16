use super::utilities::error_json::ErrorJson;
use super::utilities::request_handling;
use super::utilities::tunneling;
use crate::app_context::AppContext;
use crate::datastore::RemoteAccessType;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::rt;
use actix_web::web::{Data, Payload};
use handle_connection::handle_connection;

mod handle_connection;
mod signal_message;

pub(super) async fn open_remote_desktop_session(
    request: HttpRequest,
    context: Data<AppContext>,
    body: Payload,
) -> impl Responder {
    let session_token = match request_handling::extract_session_token(&request) {
        Ok(token) => token,
        Err(resp) => return resp,
    };

    let token = match request_handling::fetch_token(&context).await {
        Ok(t) => t,
        Err(resp) => return resp,
    };

    let session = match request_handling::fetch_session(&context, &token.jwt, &session_token).await
    {
        Ok(sess) => sess,
        Err(resp) => return resp,
    };

    if let Err(resp) =
        request_handling::ensure_session_type(&session, RemoteAccessType::RemoteDesktop)
    {
        return resp;
    }

    let Ok(device) = context
        .datastore
        .obtain_device_by_id(&token.jwt, &session.device_id, false)
        .await
    else {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Unable to retrieve device from datastore"));
    };

    if device.is_none() {
        return HttpResponse::NotFound().json(ErrorJson::from("Associated device not found"));
    }

    let device = device.unwrap();

    if !device.authorized {
        return HttpResponse::NotFound().json(ErrorJson::from("Device is unauthorized"));
    }

    let Ok(tunnel) =
        tunneling::establish_tunneled_rd(&context, &device.id, &session.instance_id).await
    else {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Failed to establish a tunnel"));
    };

    if !tunnel.is_authenticated() {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Tunnel is not authenticated"));
    }

    let (response, ws_session, ws_stream) =
        match request_handling::upgrade_to_websocket(request, body) {
            Ok(r) => r,
            Err(resp) => return resp,
        };

    rt::spawn(handle_connection(ws_stream, ws_session, tunnel));

    response
}
