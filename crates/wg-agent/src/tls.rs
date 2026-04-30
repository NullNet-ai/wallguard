use std::time::Duration;

use rustls_pki_types::{CertificateDer, PrivateKeyDer};

use crate::config::Config;

// ---------------------------------------------------------------------------
// Debug-only: accept any server certificate (self-signed dev PKI)
// ---------------------------------------------------------------------------

#[cfg(debug_assertions)]
mod danger {
    use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
    use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
    use rustls::{DigitallySignedStruct, Error, SignatureScheme};

    #[derive(Debug)]
    pub struct NoCertVerifier;

    impl ServerCertVerifier for NoCertVerifier {
        fn verify_server_cert(
            &self,
            _end_entity:    &CertificateDer<'_>,
            _intermediates: &[CertificateDer<'_>],
            _server_name:   &ServerName<'_>,
            _ocsp_response: &[u8],
            _now:           UnixTime,
        ) -> Result<ServerCertVerified, Error> {
            Ok(ServerCertVerified::assertion())
        }

        fn verify_tls12_signature(
            &self,
            _message: &[u8],
            _cert:    &CertificateDer<'_>,
            _dss:     &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, Error> {
            Ok(HandshakeSignatureValid::assertion())
        }

        fn verify_tls13_signature(
            &self,
            _message: &[u8],
            _cert:    &CertificateDer<'_>,
            _dss:     &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, Error> {
            Ok(HandshakeSignatureValid::assertion())
        }

        fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
            rustls::crypto::aws_lc_rs::default_provider()
                .signature_verification_algorithms
                .supported_schemes()
        }
    }
}

// ---------------------------------------------------------------------------
// Debug-only: custom tower connector that uses the no-verify rustls config
// ---------------------------------------------------------------------------

#[cfg(debug_assertions)]
mod no_verify_connector {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::task::{Context, Poll};

    use hyper_util::rt::TokioIo;
    use tokio::net::TcpStream;

    pub type Io = TokioIo<tokio_rustls::client::TlsStream<TcpStream>>;

    #[derive(Clone)]
    pub struct Connector {
        tls:         tokio_rustls::TlsConnector,
        server_name: rustls_pki_types::ServerName<'static>,
    }

    impl Connector {
        pub fn new(
            mut cfg: rustls::ClientConfig,
            server_name: &str,
        ) -> anyhow::Result<Self> {
            // gRPC requires h2 ALPN.
            cfg.alpn_protocols.push(b"h2".to_vec());
            let server_name = rustls_pki_types::ServerName::try_from(server_name.to_string())
                .map_err(|e| anyhow::anyhow!("invalid server name '{server_name}': {e}"))?;
            Ok(Self {
                tls: tokio_rustls::TlsConnector::from(Arc::new(cfg)),
                server_name,
            })
        }
    }

    impl tower::Service<tonic::transport::Uri> for Connector {
        type Response = Io;
        type Error   = anyhow::Error;
        type Future  = Pin<Box<dyn Future<Output = Result<Io, anyhow::Error>> + Send>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, uri: tonic::transport::Uri) -> Self::Future {
            let tls         = self.tls.clone();
            let server_name = self.server_name.clone();
            Box::pin(async move {
                let host = uri.host().unwrap_or_default();
                let port = uri.port_u16().unwrap_or(443);
                let tcp  = TcpStream::connect((host, port)).await
                    .map_err(|e| anyhow::anyhow!("TCP connect {host}:{port}: {e}"))?;
                let stream = tls.connect(server_name, tcp).await
                    .map_err(|e| anyhow::anyhow!("TLS handshake: {e}"))?;
                Ok(TokioIo::new(stream))
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Shared cert/key loader
// ---------------------------------------------------------------------------

fn load_client_cert_and_key(
    config: &Config,
) -> anyhow::Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    let cert_pem = std::fs::read_to_string(&config.tls.device_cert)?;
    let key_pem  = std::fs::read_to_string(&config.tls.device_key)?;

    let certs = rustls_pemfile::certs(&mut std::io::Cursor::new(cert_pem.as_bytes()))
        .collect::<Result<Vec<_>, _>>()?;

    let key = rustls_pemfile::private_key(&mut std::io::Cursor::new(key_pem.as_bytes()))?
        .ok_or_else(|| anyhow::anyhow!("no private key in {}", config.tls.device_key.display()))?;

    Ok((certs, key))
}

// ---------------------------------------------------------------------------
// Rustls ClientConfig (used by QUIC + TCP-TLS tunnel transport)
// ---------------------------------------------------------------------------

/// Build a rustls `ClientConfig` for mTLS tunnel connections.
///
/// Debug builds skip server certificate verification so the dev self-signed
/// PKI can be used without distributing the CA to each agent first.
#[cfg(debug_assertions)]
pub fn build_rustls_client_config(config: &Config) -> anyhow::Result<rustls::ClientConfig> {
    let (certs, key) = load_client_cert_and_key(config)?;
    tracing::warn!("TLS server certificate verification DISABLED — debug build only");
    rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(std::sync::Arc::new(danger::NoCertVerifier))
        .with_client_auth_cert(certs, key)
        .map_err(|e| anyhow::anyhow!("client auth cert: {e}"))
}

#[cfg(not(debug_assertions))]
pub fn build_rustls_client_config(config: &Config) -> anyhow::Result<rustls::ClientConfig> {
    let (certs, key) = load_client_cert_and_key(config)?;
    let ca_pem = std::fs::read_to_string(&config.tls.ca_cert)?;
    let mut root_store = rustls::RootCertStore::empty();
    for cert in rustls_pemfile::certs(&mut std::io::Cursor::new(ca_pem.as_bytes())) {
        root_store.add(cert?)?;
    }
    rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_client_auth_cert(certs, key)
        .map_err(|e| anyhow::anyhow!("client auth cert: {e}"))
}

// ---------------------------------------------------------------------------
// gRPC channel builder (used by control_channel + pipeline/transmit)
// ---------------------------------------------------------------------------

/// Build a connected tonic gRPC `Channel` with mTLS.
///
/// Debug builds use a custom connector that skips server certificate
/// verification; release builds use tonic's standard `ClientTlsConfig`.
#[cfg(debug_assertions)]
pub async fn build_grpc_channel(
    config: &Config,
    uri:    String,
) -> anyhow::Result<tonic::transport::Channel> {
    let rustls_cfg = build_rustls_client_config(config)?;
    let connector  = no_verify_connector::Connector::new(rustls_cfg, &config.server.name)?;
    tonic::transport::Channel::from_shared(uri)?
        .connect_timeout(Duration::from_secs(10))
        .connect_with_connector(connector)
        .await
        .map_err(|e| anyhow::anyhow!("gRPC connect: {e}"))
}

#[cfg(not(debug_assertions))]
pub async fn build_grpc_channel(
    config: &Config,
    uri:    String,
) -> anyhow::Result<tonic::transport::Channel> {
    use tonic::transport::{Certificate, ClientTlsConfig, Identity};
    let cert = std::fs::read_to_string(&config.tls.device_cert)?;
    let key  = std::fs::read_to_string(&config.tls.device_key)?;
    let ca   = std::fs::read_to_string(&config.tls.ca_cert)?;
    let tls  = ClientTlsConfig::new()
        .domain_name(&config.server.name)
        .identity(Identity::from_pem(&cert, &key))
        .ca_certificate(Certificate::from_pem(ca));
    tonic::transport::Channel::from_shared(uri)?
        .tls_config(tls)?
        .connect_timeout(Duration::from_secs(10))
        .connect()
        .await
        .map_err(|e| anyhow::anyhow!("gRPC connect: {e}"))
}
