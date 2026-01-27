use actix_web::{
    HttpRequest, HttpResponse, Responder,
    web::{Data, Json},
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    app_context::AppContext,
    datastore::SshSessionModel,
    http_proxy::utilities::{authorization, error_json::ErrorJson},
    utilities,
};

#[derive(Deserialize)]
pub(in crate::http_proxy) struct RequestPayload {
    tunnel_id: String,
    device_id: String,
    instance_id: String,
    username: String,
}

pub async fn create_ssh_session(
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

    let Ok(Some(service)) = context
        .datastore
        .obtain_service(&jwt, &tunnel.service_id, false)
        .await
    else {
        return HttpResponse::NotFound().json(ErrorJson::from("Service not found"));
    };

    let passphrase = utilities::random::generate_random_string(16);

    let Ok((public_key, private_key)) =
        utilities::ssh::generate_keypair(Some(passphrase.clone()), None).await
    else {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Failed to generate SSH keypair"));
    };

    let session = SshSessionModel {
        tunnel_id: tunnel.id,
        device_id: tunnel.device_id,
        instance_id: body.instance_id.clone(),
        local_addr: service.address,
        local_port: service.port,
        username: body.username.clone(),
        public_key,
        private_key,
        passphrase,
        ..Default::default()
    };

    let Ok(session_id) = context.datastore.create_ssh_session(&jwt, &session).await else {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Failed to create session"));
    };

    HttpResponse::Ok().json(json!({"session_id": session_id}))
}
