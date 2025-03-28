mod convert_request;
mod convert_response;

use convert_request::convert_request;
use convert_response::convert_response;

use actix_web::{error::ErrorServiceUnavailable, HttpRequest, HttpResponse, Result as ActixResult};
use std::net::SocketAddr;

pub async fn proxy_request(
    request: HttpRequest,
    body: actix_web::web::Payload,
    target: SocketAddr,
) -> ActixResult<HttpResponse> {
    let request = convert_request(request, body, target).await?;

    let stream = tokio::net::TcpStream::connect(target)
        .await
        .map_err(ErrorServiceUnavailable)?;

    // Write token bytes here

    let io = hyper_util::rt::TokioIo::new(stream);

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
