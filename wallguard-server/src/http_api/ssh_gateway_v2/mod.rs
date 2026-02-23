use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::rt;
use actix_web::web::{Data, Payload};

use super::utilities::error_json::ErrorJson;
use super::utilities::request_handling;
use crate::app_context::AppContext;
use crate::http_api::ssh_gateway_v2::websocket_relay::websocket_relay;
use crate::tunneling::tunnel_common::WallguardTunnel;

mod websocket_relay;

pub(super) async fn open_ssh_session(
    request: HttpRequest,
    context: Data<AppContext>,
    body: Payload,
) -> impl Responder {
    let tunnel_id = match request_handling::extract_session_token(&request) {
        Ok(tunnel_id) => tunnel_id.to_ascii_uppercase(),
        Err(response) => return response,
    };

    let Some(WallguardTunnel::Ssh(ssh_tunnel)) = context.tunnels_manager.get(&tunnel_id).await
    else {
        return HttpResponse::NotFound().json(ErrorJson::from("Tunnel not found"));
    };

    let (response, ws_session, stream) = match request_handling::upgrade_to_websocket(request, body)
    {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    rt::spawn(websocket_relay(stream, ws_session, ssh_tunnel));

    response
}
