mod db;

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

    // ---------------------------------------------------------------------------
    // Database — connect and migrate.
    // Exits immediately on failure; a partially migrated database is unusable.
    // ---------------------------------------------------------------------------
    let _pool = db::create_pool(&database_url).await.unwrap_or_else(|e| {
        tracing::error!("{e}");
        std::process::exit(1);
    });

    tracing::info!("wg-server started (stub — subsystems added in later phases)");

    // Subsystems wired up in subsequent phases:
    //   Phase 3: PKI/CA, auth, RBAC middleware
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
