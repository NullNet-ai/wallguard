mod cert_verifier;
mod convert_request;
mod convert_response;

use super::auth::authenticate;
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, error::ErrorServiceUnavailable};
use cert_verifier::NoCertificateVerification;
use convert_request::convert_request;
use convert_response::convert_response;
use hyper::rt::{Read, Write};
use hyper_util::rt::TokioIo;
use rustls::pki_types::ServerName;
use std::{net::SocketAddr, sync::Arc};
use tokio_rustls::TlsConnector;

trait ReadWrite: Read + Write {}
impl<T: Read + Write> ReadWrite for T {}

pub async fn proxy_request(
    request: HttpRequest,
    body: actix_web::web::Payload,
    target: SocketAddr,
    auth_token: Option<String>,
    is_https: bool,
) -> ActixResult<HttpResponse> {
    let request = convert_request(request, body, target).await?;

    let mut tcp_stream = tokio::net::TcpStream::connect(target)
        .await
        .map_err(ErrorServiceUnavailable)?;

    if auth_token.is_some() {
        authenticate(&mut tcp_stream, &auth_token.unwrap()).await?;
    }

    let io: Box<dyn ReadWrite + Send + Unpin> = if is_https {
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let mut config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        config
            .dangerous()
            .set_certificate_verifier(Arc::new(NoCertificateVerification));

        let connector = TlsConnector::from(Arc::new(config));
        let domain = ServerName::try_from("dummy.com").unwrap();

        let stream = connector.connect(domain, tcp_stream).await?;

        Box::new(TokioIo::new(stream))
    } else {
        Box::new(TokioIo::new(tcp_stream))
    };

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
