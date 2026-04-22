mod auth;
mod db;
mod error;
mod grpc;
mod middleware;
mod pki;
mod routes;

use std::{net::SocketAddr, sync::Arc};

use auth::JwtService;
use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use pki::Ca;
use sqlx::PgPool;
use tonic::transport::{Identity, Server as TonicServer, ServerTlsConfig};

use crate::{
    grpc::provisioning::{ProvisioningServer, ProvisioningService},
    middleware::{auth::auth_middleware, request_id::request_id_middleware},
    routes::installation_codes,
};

/// Shared application state threaded through all axum handlers.
///
/// `Ca` is wrapped in `Arc` because `rcgen::KeyPair` is not `Clone`.
#[derive(Clone)]
pub struct AppState {
    pub pool:        PgPool,
    pub ca:          Arc<Ca>,
    pub jwt:         JwtService,
    /// PEM of the Intermediate CA cert — returned to agents on enrollment.
    pub ca_cert_pem: String,
}

#[tokio::main]
async fn main() {
    // ---------------------------------------------------------------------------
    // Tracing
    // ---------------------------------------------------------------------------
    let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".into());
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,wg_server=debug".into()),
        )
        .with_target(true);
    if log_format == "json" {
        subscriber.json().init();
    } else {
        subscriber.init();
    }

    // ---------------------------------------------------------------------------
    // Configuration
    // ---------------------------------------------------------------------------
    let database_url    = require_env("DATABASE_URL");
    let ca_cert_path    = require_env("CA_CERT_PATH");
    let ca_key_path     = require_env("CA_KEY_PATH");
    let server_cert_path = require_env("SERVER_CERT_PATH");
    let server_key_path  = require_env("SERVER_KEY_PATH");
    let server_name      = std::env::var("SERVER_NAME").unwrap_or_else(|_| "localhost".into());
    let grpc_port: u16   = std::env::var("GRPC_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(50051);
    let http_port: u16   = std::env::var("HTTP_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);

    // ---------------------------------------------------------------------------
    // Database
    // ---------------------------------------------------------------------------
    let pool = db::create_pool(&database_url).await.unwrap_or_else(|e| {
        tracing::error!("{e}");
        std::process::exit(1);
    });

    // ---------------------------------------------------------------------------
    // PKI — Intermediate CA
    // ---------------------------------------------------------------------------
    let ca_cert_pem = std::fs::read_to_string(&ca_cert_path).unwrap_or_else(|e| {
        tracing::error!("cannot read CA cert {ca_cert_path}: {e}");
        std::process::exit(1);
    });
    let ca_key_pem = std::fs::read_to_string(&ca_key_path).unwrap_or_else(|e| {
        tracing::error!("cannot read CA key {ca_key_path}: {e}");
        std::process::exit(1);
    });
    let ca = Arc::new(Ca::load_pem(&ca_cert_pem, &ca_key_pem).unwrap_or_else(|e| {
        tracing::error!("cannot load CA: {e}");
        std::process::exit(1);
    }));

    // ---------------------------------------------------------------------------
    // Auth — JWT signing key
    // ---------------------------------------------------------------------------
    let jwt = JwtService::new(pool.clone()).await.unwrap_or_else(|e| {
        tracing::error!("cannot initialise JWT service: {e}");
        std::process::exit(1);
    });

    let state = AppState {
        pool: pool.clone(),
        ca:   ca.clone(),
        jwt,
        ca_cert_pem: ca_cert_pem.clone(),
    };

    // ---------------------------------------------------------------------------
    // gRPC — Provisioning service on grpc_port (server-TLS only, no mTLS)
    // ---------------------------------------------------------------------------
    let server_cert = std::fs::read_to_string(&server_cert_path).unwrap_or_else(|e| {
        tracing::error!("cannot read server cert {server_cert_path}: {e}");
        std::process::exit(1);
    });
    let server_key = std::fs::read_to_string(&server_key_path).unwrap_or_else(|e| {
        tracing::error!("cannot read server key {server_key_path}: {e}");
        std::process::exit(1);
    });

    let tls_identity = Identity::from_pem(&server_cert, &server_key);
    let tls_config   = ServerTlsConfig::new().identity(tls_identity);

    let prov_svc = ProvisioningServer::new(ProvisioningService {
        pool:        pool.clone(),
        ca:          ca.clone(),
        ca_cert_pem: ca_cert_pem.clone(),
        server_name: server_name.clone(),
    });

    let grpc_addr: SocketAddr = format!("[::]:{grpc_port}").parse().unwrap();
    tokio::spawn(async move {
        tracing::info!(port = grpc_port, "gRPC provisioning service listening");
        if let Err(e) = TonicServer::builder()
            .tls_config(tls_config)
            .expect("invalid TLS config")
            .add_service(prov_svc)
            .serve(grpc_addr)
            .await
        {
            tracing::error!("gRPC server error: {e}");
        }
    });

    // ---------------------------------------------------------------------------
    // HTTP — axum router
    // ---------------------------------------------------------------------------
    let protected = Router::new()
        .route("/api/v1/installation-codes",     post(installation_codes::create_installation_code))
        .route("/api/v1/installation-codes",     get(installation_codes::list_installation_codes))
        .layer(axum_middleware::from_fn_with_state(state.clone(), auth_middleware));

    let app = Router::new()
        .merge(protected)
        .layer(axum_middleware::from_fn(request_id_middleware))
        .with_state(state);

    let http_addr: SocketAddr = format!("[::]:{http_port}").parse().unwrap();
    tracing::info!(port = http_port, "HTTP server listening");

    let listener = tokio::net::TcpListener::bind(http_addr).await.unwrap_or_else(|e| {
        tracing::error!("cannot bind HTTP port {http_port}: {e}");
        std::process::exit(1);
    });
    axum::serve(listener, app).await.unwrap_or_else(|e| {
        tracing::error!("HTTP server error: {e}");
        std::process::exit(1);
    });
}

fn require_env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| {
        eprintln!("fatal: required environment variable {key} is not set");
        std::process::exit(1);
    })
}
