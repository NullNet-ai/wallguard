use crate::{
    grpc_server::server::WallGuardImpl,
    parser::msg_parser::parse_message,
    proto::wallguard::{CommonResponse, Packets},
};
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use tonic::{Request, Response};

impl WallGuardImpl {
    pub(crate) async fn handle_packets_impl(
        &self,
        request: Request<Packets>,
    ) -> Result<Response<CommonResponse>, Error> {
        let packets = request.into_inner();
        let (jwt_token, token_info) = Self::authenticate(packets.auth.clone())?;

        let parsed_message = parse_message(packets, &token_info);
        if parsed_message.records.is_empty() {
            return Err("No valid packets in the message").handle_err(location!());
        };

        let response = self
            .datastore
            .packets_insert(&jwt_token, parsed_message)
            .await?;

        Ok(Response::new(CommonResponse {
            message: format!("Successfully inserted {} packets", response.count),
        }))
    }
}
