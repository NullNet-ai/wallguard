mod auth;
mod command_tracker;
mod connection_registry;
mod db;
mod error;
mod grpc;
mod heartbeat;
mod middleware;
mod pki;
pub(crate) mod proto;
mod routes;
mod tunnel;

use std::{net::SocketAddr, sync::Arc, time::Duration};

use auth::JwtService;
use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use pki::Ca;
use sqlx::PgPool;
use tonic::transport::{Certificate, Identity, Server as TonicServer, ServerTlsConfig};

use crate::{
    command_tracker::CommandTracker,
    connection_registry::ConnectionRegistry,
    grpc::{
        control::{ControlServer, ControlService},
        data::{DataServer, DataSvc},
        provisioning::{ProvisioningServer, ProvisioningService},
    },
    middleware::{auth::auth_middleware, request_id::request_id_middleware},
    proto::control::{server_message, ServerMessage, ShutdownImminent},
    routes::installation_codes,
    tunnel::TunnelRegistry,
};

// ---------------------------------------------------------------------------
// Shared application state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    pub pool:           PgPool,
    pub ca:             Arc<Ca>,
    pub jwt:            JwtService,
    pub ca_cert_pem:    String,
    pub registry:       ConnectionRegistry,
    pub tracker:        CommandTracker,
    pub tunnel_registry: TunnelRegistry,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    // ── Tracing ──────────────────────────────────────────────────────────────
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

    // ── Configuration ─────────────────────────────────────────────────────────
    let database_url      = require_env("DATABASE_URL");
    let ca_cert_path      = require_env("CA_CERT_PATH");
    let ca_key_path       = require_env("CA_KEY_PATH");
    let server_cert_path  = require_env("SERVER_CERT_PATH");
    let server_key_path   = require_env("SERVER_KEY_PATH");
    let server_name       = std::env::var("SERVER_NAME").unwrap_or_else(|_| "localhost".into());
    let prov_port: u16    = env_port("GRPC_PORT",         50051);
    let control_port: u16 = env_port("CONTROL_GRPC_PORT", 50052);
    let http_port: u16    = env_port("HTTP_PORT",         8080);
    let quic_port: u16    = env_port("QUIC_PORT",         7777);
    let tcp_port: u16     = env_port("TCP_TLS_PORT",      7778);

    // ── Database ──────────────────────────────────────────────────────────────
    let pool = db::create_pool(&database_url).await.unwrap_or_else(|e| {
        tracing::error!("{e}");
        std::process::exit(1);
    });

    // ── PKI ───────────────────────────────────────────────────────────────────
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

    // ── TLS ───────────────────────────────────────────────────────────────────
    let server_cert = std::fs::read_to_string(&server_cert_path).unwrap_or_else(|e| {
        tracing::error!("cannot read server cert {server_cert_path}: {e}");
        std::process::exit(1);
    });
    let server_key = std::fs::read_to_string(&server_key_path).unwrap_or_else(|e| {
        tracing::error!("cannot read server key {server_key_path}: {e}");
        std::process::exit(1);
    });

    // ── Auth ──────────────────────────────────────────────────────────────────
    let jwt = JwtService::new(pool.clone()).await.unwrap_or_else(|e| {
        tracing::error!("cannot initialise JWT service: {e}");
        std::process::exit(1);
    });

    // ── Shared state ──────────────────────────────────────────────────────────
    let registry        = ConnectionRegistry::new();
    let tracker         = CommandTracker::new();
    let tunnel_registry = TunnelRegistry::new();
    let sweeper         = tracker.start_sweeper();

    let state = AppState {
        pool: pool.clone(),
        ca:   ca.clone(),
        jwt,
        ca_cert_pem:     ca_cert_pem.clone(),
        registry:        registry.clone(),
        tracker:         tracker.clone(),
        tunnel_registry: tunnel_registry.clone(),
    };

    // ── Tunnel listeners (QUIC :7777 + TCP-TLS :7778) ────────────────────────
    tunnel::listener::spawn_listeners(
        tunnel_registry,
        ca_cert_pem.clone(),
        server_cert.clone(),
        server_key.clone(),
        quic_port,
        tcp_port,
    );

    // ── Provisioning gRPC (server-TLS only, port 50051) ───────────────────────
    let prov_tls_config = ServerTlsConfig::new()
        .identity(Identity::from_pem(&server_cert, &server_key));

    let prov_svc = ProvisioningServer::new(ProvisioningService {
        pool:        pool.clone(),
        ca:          ca.clone(),
        ca_cert_pem: ca_cert_pem.clone(),
        server_name: server_name.clone(),
    });

    let prov_addr: SocketAddr = format!("[::]:{prov_port}").parse().unwrap();
    tokio::spawn(async move {
        tracing::info!(port = prov_port, "Provisioning gRPC listening");
        if let Err(e) = TonicServer::builder()
            .tls_config(prov_tls_config)
            .expect("invalid TLS config")
            .add_service(prov_svc)
            .serve(prov_addr)
            .await
        {
            tracing::error!("Provisioning gRPC error: {e}");
        }
    });

    // ── Control gRPC (mTLS, port 50052) ──────────────────────────────────────
    let control_tls = ServerTlsConfig::new()
        .identity(Identity::from_pem(&server_cert, &server_key))
        .client_ca_root(Certificate::from_pem(&ca_cert_pem));

    let control_svc = ControlServer::new(ControlService { state: state.clone() });
    let data_svc    = DataServer::new(DataSvc { pool: pool.clone() });

    let control_addr: SocketAddr = format!("[::]:{control_port}").parse().unwrap();
    tokio::spawn(async move {
        tracing::info!(port = control_port, "Control+Data gRPC listening (mTLS)");
        if let Err(e) = TonicServer::builder()
            .tls_config(control_tls)
            .expect("invalid mTLS config")
            .add_service(control_svc)
            .add_service(data_svc)
            .serve(control_addr)
            .await
        {
            tracing::error!("Control gRPC error: {e}");
        }
    });

    // ── HTTP (port 8080) ──────────────────────────────────────────────────────
    let protected = Router::new()
        .route("/api/v1/installation-codes", post(installation_codes::create_installation_code))
        .route("/api/v1/installation-codes", get(installation_codes::list_installation_codes))
        .layer(axum_middleware::from_fn_with_state(state.clone(), auth_middleware));

    let app = Router::new()
        .merge(protected)
        .layer(axum_middleware::from_fn(request_id_middleware))
        .with_state(state.clone());

    let http_addr: SocketAddr = format!("[::]:{http_port}").parse().unwrap();
    tracing::info!(port = http_port, "HTTP server listening");

    let listener = tokio::net::TcpListener::bind(http_addr).await.unwrap_or_else(|e| {
        tracing::error!("cannot bind HTTP port {http_port}: {e}");
        std::process::exit(1);
    });

    // ── Graceful shutdown ─────────────────────────────────────────────────────
    let shutdown = graceful_shutdown(state.registry.clone(), state.tracker.clone());

    tokio::select! {
        result = axum::serve(listener, app).with_graceful_shutdown(async { shutdown.await }) => {
            if let Err(e) = result { tracing::error!("HTTP server error: {e}"); }
        }
    }

    sweeper.abort();
    tracing::info!("wg-server stopped");
}

/// Waits for SIGTERM or SIGINT, broadcasts `ShutdownImminent` to all agents,
/// waits up to 5 s for connections to drain, then times out all pending
/// commands.  Returns when the HTTP server should shut down.
async fn graceful_shutdown(registry: ConnectionRegistry, tracker: CommandTracker) {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = signal(SignalKind::terminate())
            .expect("failed to register SIGTERM handler");
        tokio::select! {
            _ = sigterm.recv()          => tracing::info!("SIGTERM received"),
            _ = tokio::signal::ctrl_c() => tracing::info!("SIGINT received"),
        }
    }
    #[cfg(not(unix))]
    { let _ = tokio::signal::ctrl_c().await; }

    tracing::info!("initiating graceful shutdown");

    // 1. Tell all agents to reconnect in 3 s.
    registry.broadcast(ServerMessage {
        message: Some(server_message::Message::ShutdownImminent(ShutdownImminent {
            reconnect_after_ms: 3_000,
        })),
    }).await;

    // 2. Wait up to 5 s for connections to drop.
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline && !registry.is_empty().await {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // 3. Timeout all pending commands so HTTP handlers return 504.
    tracker.timeout_all().await;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn require_env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| {
        eprintln!("fatal: required environment variable {key} is not set");
        std::process::exit(1);
    })
}

fn env_port(key: &str, default: u16) -> u16 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
