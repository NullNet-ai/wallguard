mod auth;
mod db;
mod middleware;
mod pki;

use std::sync::Arc;

use auth::JwtService;
use pki::Ca;
use sqlx::PgPool;

/// Shared application state threaded through all axum handlers.
///
/// Stored in `Arc` internally by axum's `State` extractor.
/// `Ca` is wrapped in `Arc` because `rcgen::KeyPair` is not `Clone`.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub ca:   Arc<Ca>,
    pub jwt:  JwtService,
}

#[tokio::main]
async fn main() {
    // ---------------------------------------------------------------------------
    // Tracing — JSON in production, pretty in dev.
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
    // Configuration from environment.
    // ---------------------------------------------------------------------------
    let database_url = require_env("DATABASE_URL");
    let ca_cert_path = require_env("CA_CERT_PATH");
    let ca_key_path  = require_env("CA_KEY_PATH");

    // ---------------------------------------------------------------------------
    // Database — connect and run pending migrations.
    // ---------------------------------------------------------------------------
    let pool = db::create_pool(&database_url).await.unwrap_or_else(|e| {
        tracing::error!("{e}");
        std::process::exit(1);
    });

    // ---------------------------------------------------------------------------
    // PKI — load Intermediate CA for device cert signing.
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
    // Auth — load or generate JWT signing key.
    // ---------------------------------------------------------------------------
    let jwt = JwtService::new(pool.clone()).await.unwrap_or_else(|e| {
        tracing::error!("cannot initialise JWT service: {e}");
        std::process::exit(1);
    });

    let _state = AppState { pool, ca, jwt };

    tracing::info!("wg-server started — phase 3 security foundation ready");

    // Subsystems wired up in subsequent phases:
    //   Phase 4: Provisioning gRPC service
    //   Phase 5: Agent state machine stubs (build only)
    //   Phase 6: Control gRPC, connection registry, command tracker
    //   Phase 8: Tunnel sessions (QUIC endpoint, SSH, TTY, HTTP, remote desktop)
    //   Phase 9: HTTP API router (axum), SSE, WebSocket relay
}

fn require_env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| {
        eprintln!("fatal: required environment variable {key} is not set");
        std::process::exit(1);
    })
}
