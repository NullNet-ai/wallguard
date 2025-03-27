use std::net::SocketAddr;

use http_body_util::BodyExt;

async fn convert_original_request(
    request: actix_web::HttpRequest,
    body: actix_web::web::Payload,
    target: SocketAddr,
) -> Result<hyper::Request<http_body_util::Full<hyper::body::Bytes>>, actix_web::Error> {
    let uri: hyper::Uri = format!("http://{}{}", target, request.uri())
        .parse()
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let method = match *request.method() {
        actix_web::http::Method::CONNECT => hyper::Method::CONNECT,
        actix_web::http::Method::DELETE => hyper::Method::DELETE,
        actix_web::http::Method::GET => hyper::Method::GET,
        actix_web::http::Method::HEAD => hyper::Method::HEAD,
        actix_web::http::Method::OPTIONS => hyper::Method::OPTIONS,
        actix_web::http::Method::PATCH => hyper::Method::PATCH,
        actix_web::http::Method::POST => hyper::Method::POST,
        actix_web::http::Method::PUT => hyper::Method::PUT,
        actix_web::http::Method::TRACE => hyper::Method::TRACE,
        _ => hyper::Method::GET,
    };

    let mut request_builder = hyper::Request::builder().method(method).uri(uri);

    for (header_name, header_value) in request.headers() {
        if let (Ok(name), Ok(value)) = (
            hyper::header::HeaderName::from_bytes(header_name.as_str().as_bytes()),
            hyper::header::HeaderValue::from_bytes(header_value.as_bytes()),
        ) {
            request_builder = request_builder.header(name, value);
        }
    }

    let bb = hyper::body::Bytes::from(body.to_bytes().await?);
    let body = http_body_util::Full::new(bb);

    let request = request_builder
        .body(body) // Directly use the body as Bytes
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(request)
}

async fn conver_forward_response(
    mut response: hyper::Response<hyper::body::Incoming>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let response_status = actix_web::http::StatusCode::from_u16(response.status().as_u16())
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let mut response_builder = actix_web::HttpResponse::build(response_status);

    for (name, value) in response.headers().iter() {
        response_builder.insert_header((name.as_str(), value.as_bytes()));
    }

    let mut data = vec![];

    while let Some(next) = response.body_mut().frame().await {
        let frame = next.map_err(actix_web::error::ErrorInternalServerError)?;

        if let Some(chunk) = frame.data_ref() {
            data.extend_from_slice(chunk.iter().as_slice());
        }
    }

    Ok(response_builder.body(data))
}

pub async fn proxy_http_request(
    request: actix_web::HttpRequest,
    body: actix_web::web::Payload,
    target: SocketAddr,
) -> actix_web::Result<actix_web::HttpResponse> {
    let request = convert_original_request(request, body, target).await?;

    let stream = tokio::net::TcpStream::connect(target)
        .await
        .map_err(actix_web::error::ErrorServiceUnavailable)?;

    // Write token bytes here

    let io = hyper_util::rt::TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .map_err(actix_web::error::ErrorServiceUnavailable)?;

    tokio::spawn(async move {
        log::debug!("HTTP Proxy: Http session started");
        match conn.await {
            Ok(_) => log::debug!("HTTP Proxy: Http session completed"),
            Err(_) => log::error!("HTTP Proxy: Http session failed"),
        }
    });

    let response = sender
        .send_request(request)
        .await
        .map_err(actix_web::error::ErrorServiceUnavailable)?;

        log::debug!("{:?}", &response);

    let response = conver_forward_response(response).await?;

    Ok(response)
}
