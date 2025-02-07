use tonic::{Request, Response, Status};

use crate::{
    grpc_server::server::WallGuardImpl,
    parser::msg_parser::parse_message,
    proto::wallguard::{CommonResponse, Packets},
};

impl WallGuardImpl {
    pub(crate) async fn handle_packets_impl(
        &self,
        request: Request<Packets>,
    ) -> Result<Response<CommonResponse>, Status> {
        let packets = request.into_inner();
        let (jwt_token, token_info) = Self::authenticate(packets.auth.clone())?;

        let parsed_message = parse_message(packets, &token_info);
        if parsed_message.records.is_empty() {
            return Err(Status::internal("No valid packets in the message"));
        };

        let response = self
            .datastore
            .as_ref()
            .ok_or_else(|| Status::internal("Datastore is unavailable"))?
            .packets_insert(&jwt_token, parsed_message)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if !response.success {
            return Err(Status::internal(format!(
                "Status: {}, Message: {}, Error: {}",
                response.status_code, response.message, response.error
            )));
        }

        Ok(Response::new(CommonResponse {
            success: response.success,
            message: response.message,
        }))
    }
}
