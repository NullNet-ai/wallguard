use std::sync::Arc;

use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::rt;
use actix_web::web::{Data, Payload};
use tokio::sync::Mutex;

use super::utilities::error_json::ErrorJson;
use super::utilities::request_handling;
use super::utilities::tunneling;
use crate::app_context::AppContext;
use crate::datastore::SshSessionStatus;
use crate::http_proxy::ssh_gateway_v2::session::Session;
use crate::http_proxy::ssh_gateway_v2::websocket_relay::websocket_relay;
use crate::reverse_tunnel::TunnelAdapter;

mod handler;
mod internal_relay;
pub mod manager;
mod session;
mod websocket_relay;

pub(super) async fn open_ssh_session(
    request: HttpRequest,
    context: Data<AppContext>,
    body: Payload,
) -> impl Responder {
    let session_id = match request_handling::extract_session_token(&request) {
        Ok(token) => token,
        Err(response) => return response,
    };

    let token = match request_handling::fetch_token(&context).await {
        Ok(t) => t,
        Err(resp) => return resp,
    };

    let Ok(Some(session)) = context
        .datastore
        .obtain_ssh_session(&token.jwt, &session_id, false)
        .await
    else {
        return HttpResponse::NotFound().json(ErrorJson::from("Failed to obtain ssh session data"));
    };

    if !matches!(session.session_status, SshSessionStatus::Active) {
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

    let Ok(tunnel) = tunneling::establish_tunneled_ssh(
        &context,
        &device.id,
        &session.instance_id,
        &session.public_key,
    )
    .await
    else {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Failed to establish a tunnel"));
    };

    let Ok(tunnel_adapter) = TunnelAdapter::try_from(tunnel) else {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Failed to adapt tunnel transport"));
    };

    let session_intance = {
        if let Some(sesh) = context.ssh_sessions_manager.get_by_id(&session_id).await {
            sesh
        } else {
            let Ok(sesh) = Session::new(context.get_ref().clone(), tunnel_adapter, &session).await
            else {
                return HttpResponse::InternalServerError()
                    .json(ErrorJson::from("Failed to establish ssh session"));
            };

            let sesh = Arc::new(Mutex::new(sesh));

            context
                .ssh_sessions_manager
                .add(session_id, sesh.clone())
                .await;

            sesh
        }
    };

    let (response, ws_session, stream) = match request_handling::upgrade_to_websocket(request, body)
    {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    rt::spawn(websocket_relay(stream, ws_session, session_intance));

    response
}
