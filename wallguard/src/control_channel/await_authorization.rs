use nullnet_liberror::{Error, ErrorHandler, Location, location};
use wallguard_common::protobuf::wallguard_commands::{
    AuthorizationRequest, ClientMessage, client_message, server_message,
};

use crate::{
    client_data::ClientData,
    control_channel::{InboundStream, OutboundStream},
    storage::{Secret, Storage},
};

pub enum Verdict {
    Approved,
    Rejected,
}

pub async fn await_authorization(
    inbound: InboundStream,
    outbound: OutboundStream,
    client_data: ClientData,
    installation_code: impl Into<String>,
) -> Result<Verdict, Error> {
    let message = ClientMessage {
        message: Some(client_message::Message::AuthorizationRequest(
            AuthorizationRequest {
                uuid: client_data.uuid,
                code: installation_code.into(),
                category: client_data.category,
                r#type: client_data.platform.to_string(),
                target_os: client_data.target_os.to_string(),
            },
        )),
    };

    outbound
        .lock()
        .await
        .send(message)
        .await
        .handle_err(location!())?;

    let message = inbound
        .lock()
        .await
        .message()
        .await
        .handle_err(location!())?
        .ok_or("Server sent an empty message")
        .handle_err(location!())?
        .message
        .ok_or("Malformed message (empty payload)")
        .handle_err(location!())?;

    match message {
        server_message::Message::DeviceAuthorizedMessage(data) => {
            if let Some(app_id) = data.app_id {
                Storage::set_value(Secret::AppId, &app_id).await?;
            }

            if let Some(app_secret) = data.app_secret {
                Storage::set_value(Secret::AppSecret, &app_secret).await?;
            }

            Ok(Verdict::Approved)
        }
        server_message::Message::AuthorizationRejectedMessage(_) => Ok(Verdict::Rejected),
        _ => Err("Unexpected message").handle_err(location!())?,
    }
}
