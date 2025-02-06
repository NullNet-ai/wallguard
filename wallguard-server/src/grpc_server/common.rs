use chrono::Utc;
use tonic::Request;

use super::server::WallGuardImpl;

impl WallGuardImpl {
    pub(crate) fn log_request<T>(request: &Request<T>, endpoint: &str) {
        let address = request
            .remote_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or("Unknown".to_string());
        println!(
            "[{}] Request from {} to endpoint {}",
            Utc::now().to_rfc3339(),
            address,
            endpoint
        );
    }
}
