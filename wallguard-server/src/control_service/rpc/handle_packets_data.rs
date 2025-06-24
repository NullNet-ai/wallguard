use crate::control_service::service::WallGuardService;
use crate::protocol::wallguard_service::PacketsData;
use crate::traffic_handler::msg_parser::parse_message;
use nullnet_libtoken::Token;
use tonic::{Request, Response, Status};

impl WallGuardService {
    pub(crate) async fn handle_packets_data_impl(
        &self,
        request: Request<PacketsData>,
    ) -> Result<Response<()>, Status> {
        let data = request.into_inner();

        let token =
            Token::from_jwt(&data.token).map_err(|_| Status::internal("Malformed JWT token"))?;

        let _ = self
            .ensure_device_exists_and_authrorized(&token)
            .await
            .map_err(|err| Status::internal(err.to_str()))?;

        let packets_number = data.packets.len();

        let parsed_message = parse_message(data, &token, &self.ip_info_tx);

        log::info!(
            "Received {} packets. Parsed {} connections",
            packets_number,
            parsed_message.records.len()
        );

        if !parsed_message.records.is_empty() {
            self.context
                .datastore
                .create_connections(&token.jwt, parsed_message)
                .await
                .map_err(|_| Status::internal("Datastore operation failed"))?;
        }

        Ok(Response::new(()))
    }
}
