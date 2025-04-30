#![allow(clippy::module_name_repetitions)]

use crate::grpc_server::{ADDR, PORT};
use crate::utils::{ACCOUNT_ID, ACCOUNT_SECRET};
use app_context::AppContext;
use clap::Parser;
use nullnet_liberror::Error;
use std::time::Duration;
use tokio::signal;

mod app_context;
mod cli;
mod client_stream;
mod datastore;
mod grpc_server;
mod http_server;
mod parser;
mod proto;
mod tunnel;
mod utils;

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let datastore_logger_config = nullnet_liblogging::DatastoreConfig::new(
        ACCOUNT_ID.as_str(),
        ACCOUNT_SECRET.as_str(),
        ADDR,
        PORT,
    );
    let logger_config =
        nullnet_liblogging::LoggerConfig::new(true, false, Some(datastore_logger_config), vec![]);
    nullnet_liblogging::Logger::init(logger_config);

    let app_context = AppContext::new()
        .await
        .expect("Failed to initialize AppContext");

    let _ = terminate_active_rm_sessions(&app_context).await;

    tokio::select! {
        _ = tunnel::monitor_idle_profiles(app_context.clone(), Duration::from_secs(60 * 30)) => {},
        _ = grpc_server::run_grpc_server(app_context.clone(), args) => {},
        _ = http_server::run_http_server(app_context) => {},
        _ = signal::ctrl_c() => {}
    };
}

async fn terminate_active_rm_sessions(context: &AppContext) -> Result<(), Error> {
    let token = context
        .datastore
        .login(ACCOUNT_ID.to_string(), ACCOUNT_SECRET.to_string())
        .await?;

    // Before starting the API, mark all active sessions in the database as terminated.
    // This ensures a clean state after an unexpected server crash or reload,
    // during which sessions might not have been properly closed.
    // By resetting all sessions at startup, we avoid inconsistencies and start fresh.
    context
        .datastore
        .device_terminate_all_active_sessions(&token)
        .await?;

    Ok(())
}
