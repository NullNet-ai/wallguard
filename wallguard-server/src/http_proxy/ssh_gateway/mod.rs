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
use relay::relay;

mod relay;
mod ssh_session;

pub(super) async fn open_ssh_session(
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

    if let Err(resp) = request_handling::ensure_session_type(&session, RemoteAccessType::Ssh) {
        return resp;
    }

    let Ok(device) = context
        .datastore
        .obtain_device_by_id(&token.jwt, &session.device_id)
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

    let keypair =
        match request_handling::fetch_ssh_keypair(&context, &token.jwt, &session.device_id).await {
            Ok(kp) => kp,
            Err(resp) => return resp,
        };

    let Ok(stream) =
        tunneling::establish_tunneled_ssh(&context, &device.uuid, &keypair.public_key).await
    else {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Failed to establish a tunnel"));
    };

    let ssh_session = match ssh_session::SSHSession::new(stream, &keypair).await {
        Ok(sess) => sess,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ErrorJson::from("Failed to establish SSH session"));
        }
    };

    let (response, ws_session, stream) = match request_handling::upgrade_to_websocket(request, body)
    {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    rt::spawn(relay(stream, ws_session, ssh_session));
    response
}
