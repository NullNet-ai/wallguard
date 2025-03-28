mod convert_message;
mod proxy_message;
mod proxy_websocket;

use std::net::SocketAddr;

use actix_web::error::ErrorServiceUnavailable;
use actix_web::{web::Payload, HttpRequest, HttpResponse};
use proxy_websocket::ProxyWebsocket;

pub async fn proxy_request(
    request: HttpRequest,
    body: Payload,
    target: SocketAddr,
) -> actix_web::Result<HttpResponse> {
    let tcp_stream = tokio::net::TcpStream::connect(target)
        .await
        .map_err(ErrorServiceUnavailable)?;

    // Wreite bytes here

    let (ws_stream, _) =
        tokio_tungstenite::client_async(format!("ws://{}{}", target, request.uri()), tcp_stream)
            .await
            .map_err(ErrorServiceUnavailable)?;

    let proxy_websocket = ProxyWebsocket::from(ws_stream);

    actix_web_actors::ws::start(proxy_websocket, &request, body)
}
