mod convert_message;
mod proxy_message;
mod proxy_websocket;
mod stop_message;

use super::auth::authenticate;
use actix_web::error::ErrorServiceUnavailable;
use actix_web::{HttpRequest, HttpResponse, web::Payload};
use proxy_websocket::ProxyWebsocket;
use std::net::SocketAddr;

pub async fn proxy_request(
    request: HttpRequest,
    body: Payload,
    target: SocketAddr,
    auth_token: Option<String>,
) -> actix_web::Result<HttpResponse> {
    let mut tcp_stream = tokio::net::TcpStream::connect(target)
        .await
        .map_err(ErrorServiceUnavailable)?;

    if auth_token.is_some() {
        authenticate(&mut tcp_stream, &auth_token.unwrap()).await?;
    }

    let (ws_stream, _) =
        tokio_tungstenite::client_async(format!("ws://{}{}", target, request.uri()), tcp_stream)
            .await
            .map_err(ErrorServiceUnavailable)?;

    let proxy_websocket = ProxyWebsocket::from(ws_stream);

    actix_web_actors::ws::start(proxy_websocket, &request, body)
}
