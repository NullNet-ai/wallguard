use std::sync::Arc;

use actix_web::{
    HttpRequest, HttpResponse, Responder,
    web::{Data, Json},
};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;

use crate::{
    app_context::AppContext,
    datastore::{TtySessionModel, TunnelType},
    http_api::utilities::{authorization, error_json::ErrorJson, tunneling},
    utilities,
};

#[derive(Deserialize)]
pub(in crate::http_api) struct RequestPayload {
    tunnel_id: String,
    device_id: String,
    instance_id: String,
    username: String,
}

pub async fn create_tty_session(
    request: HttpRequest,
    context: Data<AppContext>,
    body: Json<RequestPayload>,
) -> impl Responder {
    let Some(jwt) = authorization::extract_authorization_token(&request) else {
        return HttpResponse::Unauthorized().json(ErrorJson::from("Missing Authorization header"));
    };

    let Ok(Some(device)) = context
        .datastore
        .obtain_device_by_id(&jwt, &body.device_id, false)
        .await
    else {
        return HttpResponse::NotFound().json(ErrorJson::from("Device not found"));
    };

    if !device.authorized {
        return HttpResponse::BadRequest().json(ErrorJson::from("Device is not authorized"));
    }

    let Ok(Some(tunnel)) = context
        .datastore
        .obtain_tunnel(&jwt, &body.tunnel_id, false)
        .await
    else {
        return HttpResponse::NotFound().json(ErrorJson::from("Tunnel not found"));
    };

    if tunnel.device_id != device.id {
        return HttpResponse::BadRequest().json(ErrorJson::from("Bad device id"));
    }

    if !matches!(tunnel.tunnel_type, TunnelType::Tty) {
        return HttpResponse::BadRequest().json(ErrorJson::from("Wrong tunnel type"));
    }

    let Ok(Some(service)) = context
        .datastore
        .obtain_service(&jwt, &tunnel.service_id, false)
        .await
    else {
        return HttpResponse::NotFound().json(ErrorJson::from("Service not found"));
    };

    let mut session = TtySessionModel {
        tunnel_id: tunnel.id,
        device_id: tunnel.device_id,
        instance_id: body.instance_id.clone(),
        // username: body.username.clone(),
        ..Default::default()
    };

    let Ok(session_id) = context.datastore.create_tty_session(&jwt, &session).await else {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Failed to create session"));
    };

    session.id = session_id.clone();

    let Ok(tunnel) =
        tunneling::establish_tunneled_tty(&context, &device.id, &session.instance_id).await
    else {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Failed to establish a tunnel"));
    };

    let Ok(sesh) =
        super::super::tty_gateway_v2::Session::new(context.get_ref().clone(), tunnel, &session)
            .await
    else {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Failed to establish ssh session"));
    };

    context
        .tty_sessions_manager
        .add(session_id.clone(), Arc::new(Mutex::new(sesh)))
        .await;

    HttpResponse::Ok().json(json!({"session_id": session_id}))
}
