use crate::app_context::AppContext;
use crate::datastore::RemoteAccessSession;
use crate::datastore::RemoteAccessType;
use crate::datastore::SSHKeypair;
use crate::http_proxy::utilities::authorization;
use crate::http_proxy::utilities::error_json::ErrorJson;
use crate::token_provider::Token;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::web::Payload;
use actix_ws::{MessageStream, Session as WSSession};
use std::sync::Arc;

pub fn extract_session_token(req: &HttpRequest) -> Result<String, HttpResponse> {
    authorization::extract_proxy_session_token(req).ok_or_else(|| {
        HttpResponse::Unauthorized().json(ErrorJson::from("Session token is missing"))
    })
}

pub async fn fetch_token(ctx: &AppContext) -> Result<Arc<Token>, HttpResponse> {
    ctx.token_provider.get().await.map_err(|_| {
        HttpResponse::InternalServerError()
            .json(ErrorJson::from("Server error, can't obtain a token"))
    })
}

pub async fn fetch_session(
    ctx: &AppContext,
    jwt: &str,
    session_token: &str,
) -> Result<RemoteAccessSession, HttpResponse> {
    match ctx.datastore.obtain_session(jwt, session_token).await {
        Ok(Some(sess)) => Ok(sess),
        Ok(None) => Err(HttpResponse::NotFound().json(ErrorJson::from(format!(
            "No session with token {}",
            session_token
        )))),
        Err(_) => {
            Err(HttpResponse::InternalServerError()
                .json(ErrorJson::from("Datastore operation failed")))
        }
    }
}

pub fn ensure_session_type(
    session: &RemoteAccessSession,
    expected: RemoteAccessType,
) -> Result<(), HttpResponse> {
    if session.r#type != expected {
        Err(HttpResponse::BadRequest().json(ErrorJson::from("Wrong session type")))
    } else {
        Ok(())
    }
}

pub async fn fetch_ssh_keypair(
    ctx: &AppContext,
    jwt: &str,
    device_id: &str,
) -> Result<SSHKeypair, HttpResponse> {
    match ctx.datastore.obtain_ssh_keypair(jwt, device_id).await {
        Ok(Some(keypair)) => Ok(keypair),
        Ok(None) => {
            Err(HttpResponse::NotFound().json(ErrorJson::from("No SSH keys have been found")))
        }
        Err(_) => {
            Err(HttpResponse::InternalServerError()
                .json(ErrorJson::from("Datastore operation failed")))
        }
    }
}

pub fn upgrade_to_websocket(
    request: HttpRequest,
    body: Payload,
) -> Result<(HttpResponse, WSSession, MessageStream), HttpResponse> {
    actix_ws::handle(&request, body)
        .map_err(|err| HttpResponse::InternalServerError().json(ErrorJson::from(err.to_string())))
}
