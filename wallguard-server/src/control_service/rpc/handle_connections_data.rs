use crate::control_service::service::WallGuardService;
use crate::token::Token;
use nullnet_libipinfo::get_ip_to_lookup;
use std::net::IpAddr;
use tonic::{Request, Response, Status};
use wallguard_common::protobuf::wallguard_service::ConnectionsData;

impl WallGuardService {
    pub(crate) async fn handle_connections_data_impl(
        &self,
        request: Request<ConnectionsData>,
    ) -> Result<Response<()>, Status> {
        let data = request.into_inner();

        let token =
            Token::from_jwt(&data.token).map_err(|_| Status::internal("Malformed JWT token"))?;

        let _ = self
            .ensure_device_exists_and_authrorized(&token)
            .await
            .map_err(|err| Status::internal(err.to_str()))?;

        let connections_count = data.connections.len();

        for conn in &data.connections {
            let src: Option<IpAddr> = conn.source_ip.parse().ok();
            let dst: Option<IpAddr> = conn.destination_ip.parse().ok();
            if let (Some(src), Some(dst)) = (src, dst) {
                let _ = self.ip_info_tx.send(get_ip_to_lookup(src, dst));
            }
        }

        log::info!("Received {} pre-parsed connections", connections_count);

        if !data.connections.is_empty() {
            let start = std::time::Instant::now();
            self.context
                .datastore
                .create_connections(&token.jwt, &token, data)
                .await
                .map_err(|_| Status::internal("Datastore operation failed"))?;
            log::info!(
                "create_connections: inserted {} records in {}ms",
                connections_count,
                start.elapsed().as_millis()
            );
        }

        Ok(Response::new(()))
    }
}
