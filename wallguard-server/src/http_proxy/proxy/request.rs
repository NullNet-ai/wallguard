use actix_web::Error as ActixError;
use actix_web::HttpRequest as ActixRequest;
use actix_web::HttpResponse as ActixResponse;
use actix_web::Result as ActixResult;
use actix_web::error::ErrorInternalServerError as InternalServerError;
use actix_web::http::Method as ActixMethod;
use actix_web::http::StatusCode as ActixStatus;
use actix_web::web::Payload as ActixBody;
use http_body_util::BodyExt;
use http_body_util::Full as BodyWrapper;
use hyper::Method as HyperMethod;
use hyper::Request as HyperRequest;
use hyper::Response as HyperResponse;
use hyper::body::Bytes as HyperBody;
use hyper::body::Incoming;
use hyper::header::HeaderName as HyperHeaderName;
use hyper::header::HeaderValue as HyperHeaderValue;
use hyper::rt::{Read, Write};
use hyper_util::rt::TokioIo;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, RootCertStore};
use std::sync::Arc;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use wallguard_common::cert_verifier::AcceptAllVerifier;
use webpki_roots::TLS_SERVER_ROOTS;

use crate::http_proxy::utilities::error_json::ErrorJson;
use crate::reverse_tunnel::TunnelAdapter;

async fn handshake(
    tunnel: TunnelAdapter,
    server_name: impl Into<String>,
) -> Result<TlsStream<TunnelAdapter>, Error> {
    let mut root_store = RootCertStore::empty();
    root_store.extend(TLS_SERVER_ROOTS.iter().map(|ta| ta.to_owned()));

    let mut config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    // @TODO:
    // Fetch target certificates and validate them.
    config
        .dangerous()
        .set_certificate_verifier(Arc::new(AcceptAllVerifier));

    let connector = TlsConnector::from(Arc::new(config));

    let domain = ServerName::try_from(server_name.into()).handle_err(location!())?;

    let tls_stream = connector
        .connect(domain, tunnel)
        .await
        .handle_err(location!())?;

    Ok(tls_stream)
}

fn convert_method(method: &ActixMethod) -> HyperMethod {
    match *method {
        ActixMethod::CONNECT => HyperMethod::CONNECT,
        ActixMethod::DELETE => HyperMethod::DELETE,
        ActixMethod::GET => HyperMethod::GET,
        ActixMethod::HEAD => HyperMethod::HEAD,
        ActixMethod::OPTIONS => HyperMethod::OPTIONS,
        ActixMethod::PATCH => HyperMethod::PATCH,
        ActixMethod::POST => HyperMethod::POST,
        ActixMethod::PUT => HyperMethod::PUT,
        ActixMethod::TRACE => HyperMethod::TRACE,
        _ => HyperMethod::GET,
    }
}

async fn convert_request(
    request: ActixRequest,
    body: ActixBody,
    domain: &str,
) -> ActixResult<HyperRequest<BodyWrapper<HyperBody>>> {
    let uri: hyper::Uri = request
        .uri()
        .to_string()
        .parse()
        .map_err(InternalServerError)?;

    let method = convert_method(request.method());

    let mut request_builder = hyper::Request::builder().method(method).uri(uri);

    for (header_name, header_value) in request.headers() {
        if header_name.as_str() == "host" || header_name.as_str() == "referer" {
            continue;
        }

        if let (Ok(name), Ok(value)) = (
            HyperHeaderName::from_bytes(header_name.as_str().as_bytes()),
            HyperHeaderValue::from_bytes(header_value.as_bytes()),
        ) {
            request_builder = request_builder.header(name, value);
        }
    }

    request_builder = request_builder.header(hyper::header::HOST, domain);

    let body = BodyWrapper::new(body.to_bytes().await?);

    let request = request_builder.body(body).map_err(InternalServerError)?;

    Ok(request)
}

async fn convert_response(
    mut response: HyperResponse<Incoming>,
) -> Result<ActixResponse, ActixError> {
    let response_status =
        ActixStatus::from_u16(response.status().as_u16()).map_err(InternalServerError)?;

    let mut response_builder = actix_web::HttpResponse::build(response_status);

    for (name, value) in response.headers().iter() {
        response_builder.insert_header((name.as_str(), value.as_bytes()));
    }

    let mut data = vec![];

    while let Some(next) = response.body_mut().frame().await {
        let frame = next.map_err(InternalServerError)?;

        if let Some(chunk) = frame.data_ref() {
            data.extend_from_slice(chunk.iter().as_slice());
        }
    }

    Ok(response_builder.body(data))
}

pub async fn proxy_request(
    request: ActixRequest,
    body: ActixBody,
    domain: &str,
    is_https: bool,
    tunnel: TunnelAdapter,
) -> ActixResponse {
    let Ok(request) = convert_request(request, body, domain).await else {
        return ActixResponse::InternalServerError().into();
    };

    trait ReadWrite: Read + Write {}
    impl<T: Read + Write> ReadWrite for T {}

    let io: Box<dyn ReadWrite + Send + Unpin> = if is_https {
        let Ok(tls_stream) = handshake(tunnel, domain).await else {
            return ActixResponse::ServiceUnavailable().json(ErrorJson::from("Handshake failed"));
        };
        Box::new(TokioIo::new(tls_stream))
    } else {
        Box::new(TokioIo::new(tunnel))
    };

    let Ok((mut sender, conn)) = hyper::client::conn::http1::handshake(io).await else {
        return ActixResponse::ServiceUnavailable().into();
    };

    tokio::spawn(conn);

    let Ok(response) = sender.send_request(request).await else {
        return ActixResponse::ServiceUnavailable().into();
    };

    let Ok(response) = convert_response(response).await else {
        return ActixResponse::InternalServerError().into();
    };

    response
}
