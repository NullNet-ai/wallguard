use crate::proto::wallguard::wall_guard_client::WallGuardClient;
pub use crate::proto::wallguard::{Empty, Packet, Packets};
use tonic::transport::{Certificate, Channel, ClientTlsConfig};
use tonic::Request;

mod proto;

#[derive(Clone)]
pub struct WallGuardGrpcInterface {
    client: WallGuardClient<Channel>,
}

static CA_CERT: once_cell::sync::Lazy<Certificate> = once_cell::sync::Lazy::new(|| {
    Certificate::from_pem(
        std::fs::read_to_string("tls/ca.pem").expect("Failed to read CA certificate"),
    )
});

impl WallGuardGrpcInterface {
    #[allow(clippy::missing_panics_doc)]
    pub async fn new(addr: &'static str, port: u16) -> Self {
        let tls = ClientTlsConfig::new().ca_certificate(CA_CERT.to_owned());

        let Ok(channel) = Channel::from_shared(format!("https://{addr}:{port}"))
            .expect("Failed to parse address")
            .tls_config(tls)
            .expect("Failed to configure up TLS")
            .connect()
            .await
        else {
            println!("Failed to connect to the server. Retrying in 1 second...");
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            return Box::pin(WallGuardGrpcInterface::new(addr, port)).await;
        };

        println!("Connected to the server");

        Self {
            client: WallGuardClient::new(channel),
        }
    }

    pub async fn handle_packets(&mut self, message: Packets) -> Option<Empty> {
        self.client
            .handle_packets(Request::new(message))
            .await
            .map(tonic::Response::into_inner)
            .ok()
    }
}
