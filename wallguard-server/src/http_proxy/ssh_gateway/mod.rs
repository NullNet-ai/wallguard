use super::utilities::error_json::ErrorJson;
use super::utilities::tunneling;
use crate::app_context::AppContext;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::rt;
use actix_web::web::{Data, Payload};
use async_ssh2_lite::TokioTcpStream;
use relay::relay;

mod helpers;
mod relay;
mod ssh_session;

pub(super) async fn open_ssh_session(
    request: HttpRequest,
    context: Data<AppContext>,
    body: Payload,
) -> impl Responder {
    let session_token = match helpers::extract_session_token(&request) {
        Ok(token) => token,
        Err(resp) => return resp,
    };

    let token = match helpers::fetch_token(&context).await {
        Ok(t) => t,
        Err(resp) => return resp,
    };

    let session = match helpers::fetch_session(&context, &token.jwt, &session_token).await {
        Ok(sess) => sess,
        Err(resp) => return resp,
    };

    if let Err(resp) = helpers::ensure_session_is_ssh(&session) {
        return resp;
    }

    let keypair = match helpers::fetch_ssh_keypair(&context, &token.jwt, &session.device_id).await {
        Ok(kp) => kp,
        Err(resp) => return resp,
    };

    let Ok(stream) =
        tunneling::establish_tunneled_ssh(&context, &session.device_id, &keypair.public_key).await
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

    let (response, ws_session, stream) = match helpers::upgrade_to_websocket(request, body) {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    rt::spawn(relay(stream, ws_session, ssh_session));
    response
}
