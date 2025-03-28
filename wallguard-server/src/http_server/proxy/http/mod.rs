mod convert_request;
mod convert_response;

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, error::ErrorServiceUnavailable};
use convert_request::convert_request;
use convert_response::convert_response;
use std::net::SocketAddr;

use super::auth::authenticate;

pub async fn proxy_request(
    request: HttpRequest,
    body: actix_web::web::Payload,
    target: SocketAddr,
    auth_token: Option<String>,
) -> ActixResult<HttpResponse> {
    let request = convert_request(request, body, target).await?;

    let mut tcp_stream = tokio::net::TcpStream::connect(target)
        .await
        .map_err(ErrorServiceUnavailable)?;

    if auth_token.is_some() {
        authenticate(&mut tcp_stream, &auth_token.unwrap()).await?;
    }

    let io = hyper_util::rt::TokioIo::new(tcp_stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .map_err(ErrorServiceUnavailable)?;

    tokio::spawn(conn);

    let response = sender
        .send_request(request)
        .await
        .map_err(ErrorServiceUnavailable)?;

    let response = convert_response(response).await?;

    Ok(response)
}
