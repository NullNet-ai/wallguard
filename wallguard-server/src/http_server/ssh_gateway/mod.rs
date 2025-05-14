// SSH Gateway
// This module implements a WebSocket server that acts as a bridge between a browser-based UI portal
// and an SSH server. Since browsers cannot open raw SSH (TCP) connections directly, this module serves
// as the intermediary to tunnel SSH communication over WebSocket.

mod session;
mod ssh_message;
mod stop_message;

use super::common::extract_session_from_request;
use crate::app_context::AppContext;
use crate::utils::{ACCOUNT_ID, ACCOUNT_SECRET};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Payload},
};
use nullnet_libtunnel::Profile;
use session::Session;

pub(super) async fn open_ssh_session(
    request: HttpRequest,
    context: Data<AppContext>,
    body: Payload,
) -> actix_web::Result<HttpResponse> {
    let Some(session) = extract_session_from_request(&request) else {
        return Ok(HttpResponse::NotFound().body(""));
    };

    let Some(profile) = context
        .tunnel
        .lock()
        .await
        .get_profile_if_online_by_public_session_id(&session)
        .await
        .cloned()
    else {
        return Ok(HttpResponse::NotFound().body(""));
    };

    let device_id = profile.device_id();

    let Ok(token) = context
        .datastore
        .login(ACCOUNT_ID.to_string(), ACCOUNT_SECRET.to_string())
        .await
    else {
        return Ok(HttpResponse::InternalServerError().body("Datastore login failed"));
    };

    let Some(ssh_keypair) = context
        .datastore
        .fetch_ssh_keypair_for_device(&device_id, &token)
        .await
    else {
        return Ok(HttpResponse::InternalServerError()
            .body("There is no SSH keypair assosiated with the device"));
    };

    let visitor_addr = profile.get_visitor_addr();
    let visitor_token = profile.get_visitor_token();

    let Ok(session) = Session::new(visitor_addr, visitor_token, &ssh_keypair).await else {
        return Ok(HttpResponse::InternalServerError().body("Failed to create a SSH session"));
    };

    actix_web_actors::ws::start(session, &request, body)
}
