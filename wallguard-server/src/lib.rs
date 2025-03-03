use std::time::Duration;
use tonic::transport::Channel;
use tonic::Request;

use crate::proto::wallguard::wall_guard_client::WallGuardClient;
pub use crate::proto::wallguard::*;

mod proto;

#[derive(Clone)]
pub struct WallGuardGrpcInterface {
    client: WallGuardClient<Channel>,
}

// static CA_CERT: once_cell::sync::Lazy<Certificate> = once_cell::sync::Lazy::new(|| {
//     Certificate::from_pem(
//         std::fs::read_to_string("tls/ca.pem").expect("Failed to read CA certificate"),
//     )
// });

impl WallGuardGrpcInterface {
    #[allow(clippy::missing_panics_doc)]
    pub async fn new(addr: &str, port: u16) -> Self {
        // let tls = ClientTlsConfig::new().ca_certificate(CA_CERT.to_owned());
        let s = format!("http://{addr}:{port}");

        let Ok(channel) = Channel::from_shared(s)
            .expect("Failed to parse address")
            .timeout(Duration::from_secs(10))
            // .tls_config(tls)
            // .expect("Failed to configure up TLS")
            .connect()
            .await
        else {
            println!("Failed to connect to the server. Retrying in 10 seconds...");
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            return Box::pin(WallGuardGrpcInterface::new(addr, port)).await;
        };

        Self {
            client: WallGuardClient::new(channel).max_decoding_message_size(50 * 1024 * 1024),
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn login(&mut self, app_id: String, app_secret: String) -> Result<String, String> {
        let response = self
            .client
            .login(Request::new(LoginRequest { app_id, app_secret }))
            .await
            .map_err(|e| e.to_string())?;

        Ok(response.into_inner().token)
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn heartbeat(&mut self, token: String) -> Result<HeartbeatResponse, String> {
        self.client
            .heartbeat(Request::new(HeartbeatRequest {
                auth: Some(Authentication { token }),
            }))
            .await
            .map(tonic::Response::into_inner)
            .map_err(|e| e.to_string())
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn handle_packets(&mut self, message: Packets) -> Result<CommonResponse, String> {
        self.client
            .handle_packets(Request::new(message))
            .await
            .map(tonic::Response::into_inner)
            .map_err(|e| e.to_string())
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn handle_config(
        &mut self,
        message: ConfigSnapshot,
    ) -> Result<CommonResponse, String> {
        self.client
            .handle_config(Request::new(message))
            .await
            .map(tonic::Response::into_inner)
            .map_err(|e| e.to_string())
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn handle_logs(&mut self, message: Logs) -> Result<CommonResponse, String> {
        self.client
            .handle_logs(Request::new(message))
            .await
            .map(tonic::Response::into_inner)
            .map_err(|e| e.to_string())
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn setup_client(&mut self, request: SetupRequest) -> Result<CommonResponse, String> {
        self.client
            .setup(Request::new(request))
            .await
            .map(tonic::Response::into_inner)
            .map_err(|e| e.to_string())
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn device_status(&mut self, token: String) -> Result<StatusResponse, String> {
        let response = self
            .client
            .status(Request::new(StatusRequest {
                auth: Some(Authentication { token }),
            }))
            .await
            .map(tonic::Response::into_inner)
            .map_err(|e| e.to_string())?;

        Ok(response)
    }
}
