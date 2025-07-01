mod config;
mod rpc;
mod service;
mod ensure_device_exists_and_authrorized;

use crate::app_context::AppContext;
use config::ControlServiceConfig;
use service::WallGuardService;

/// Starts the control service.
///
/// The control service is the central gRPC server that agents and clients connect to.
/// If an error occurs while starting or running the server, the program will terminate,
/// as this is the most critical component of the system and cannot run in a degraded state.
pub async fn run_control_service(context: AppContext) {
    let config = ControlServiceConfig::from_env();
    log::info!("Control Service running on {}", config.addr);
    if let Err(e) = WallGuardService::new(context).serve(config.addr).await {
        log::error!("Control service failed: {}", e.to_str());
        std::process::exit(1);
    }
}
